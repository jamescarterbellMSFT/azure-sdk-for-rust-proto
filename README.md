# Azure/azure-sdk-for-net prototypes

[![ci](https://github.com/heaths/azure-sdk-for-rust-proto/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/heaths/azure-sdk-for-rust-proto/actions/workflows/ci.yml)

This repo is to basically prototype different ideas for [Azure/azure-sdk-for-rust] to create our [guidelines].
Nothing herein should be interpreted as a matter of policy.

## Examples

Find all examples under the [`sdk/`](sdk/) directory. The following are what we are focusing on currently:

1. [client_new_method_params]

   Construct clients with `new(...)` and call methods with required parameters and optional call parameters including policy customizations.

   Clients are immutable so long as the client options are cloned or copied.

   **Pros**

   * Similar feel to most other languages.
   * Can share options across multiple, distinct client method calls.

   **Cons**

   * Either callers or the callee must allocate, or we use a single lazy allocation of the `Default` implementation.
   * Callers may set ambiguous or conflicting options.

   **Decision**

   We'll proceed with guidelines using this pattern. For some clients like `BlobClient` this is important,
   but we should strive to actually share options and not clone or copy them or this will be of little value over builders.

   Options structs should derive or implement `Default` to make them future-proof barring breaking changes like adding required fields.

2. [client_builder_method_builder]

   Builders are fairly common in Rust, used in the likes of the AWS SDK (only for method calls), Bevy (ECS game engine),
   and some others. I wouldn't go so far as saying they are idiomatic, though: many projects even for cloud-related crates
   don't use them. But they do help validate or constrain inputs and can even be used with the
   [typestate builder pattern](https://gist.github.com/heaths/1eb608df947de5d5b47da0ee6a5a5c6d) to enforce
   mutually-exclusive settings or required groups of settings in ways that setting public fields or calling field setters
   cannot do.

   This prototype uses builders for both client - and, more notably, client options - as well as service method calls.

   Client builders as prototyped are mutable, but clients created from `build(&self) -> Client` are immutable.
   Client method builders are also mutable until `send(&self) -> BoxFuture<Result<Response>>` is called.

   **Pros**

   * If using builders at all, this is consistently using them for client creation and client method calls.
   * Can lead callers down the path of success using the [typestate builder pattern] when necessary.
   * Can validate options beyond type safety since setters are used.
   * Consistent with some other larger frameworks like `bevy`.

   **Cons**

   * Method builders do not support sharing common options across multiple, distinct client method calls.

   **Decision**

   There are a lot of advantages to discoverability and future-proofing, but sharing options across multiple, distinct method calls
   is difficult. Even if we could solve sharing a subset of options - which I believe we could - it would most likely
   introduce lifetime constraints that would likely make the API unwieldy to callers.

   Even if we do not end up using builders for client construction and method calls, there may still be limited use cases
   for which the [typestate builder pattern] is well-suited like for constructing SAS URLs.

3. [client_new_method_builder]

   A combination of [client_new_method_params] to construct clients using `new(...)` and [client_builder_method_builder]
   using a builder for methods to optionally configure per-call settings.

   This is similar to how the [AWS SDK](https://awslabs.github.io/aws-sdk-rust/) is defined.

   Clients are immutable so long as the client options are cloned or copied.
   Client method builds are mutable until `send(&self) -> BoxFuture<Result<Response>>` is called.

   Similar pros and cons to [client_builder_method_builder], but:

   **Pros**

   * Consistent with the AWS SDKs, so familiar call patterns may ease migration to Azure.

   **Cons**

   * Inconsistent - perhaps even jarring - to use a client "constructor" but builders for client methods.

   **Decision**

   If we're going to use builders we should be consistent. If anything, it's more likely callers would want to share options
   across multiple, distinct method calls as opposed to creating numerous clients from a shared options bag - at least with
   client-specific options.

4. [client_new_method_params_context]

   Similar to [client_new_method_params] but requires a `&Context` parameter declared first in the list of parameters.
   This is similar to Go and can be used for OpenTelemetry tracing, but is not used for canceling a `Future` - part of
   Rust's async framework. To cancel a `Future` you merely need to `drop()` it. Currently, this `Context` might only be
   a property bag.

   Clients are immutable so long as the client options are cloned or copied.

   Similar pros and cons to [client_new_method_params], but:

   **Pros**

   * For those callers who want to add context or who may expose context to their own callers when wrapping our APIs,
     this may be more obvious that something should be passed.

   **Cons**

   * Most callers will end up passing `Context::default` (by value or reference) for the first param, which is not common.
     Even passing as an `Option<Context>` means most customers will pass `None`, and having to pass multiple `None`s is uncommon.
     Comparing to other languages besides Go where `context.Context` is already idiomatic, few callers pass optional context.

   **Decision**

   Requiring an argument that the majority of callers will either pass `None` or `Context::default()` is not common.
   While having it as a required parameter does significantly improve discoverability, it still does not guarantee that a container
   wrapping our APIs will expose it to their callers.

5. [client_new_method_params_struct]

   Similar to [client_new_method_params] but all parameters, body, and client method options are defined by a struct.
   If any parameters are required e.g., URL path or query string parameters, this struct could only be created by a
   `new(...)` method. If every field is optional, it could derive or implement `Default`.

   **Pros**

   * All "parameters" are named, which makes reviewing code and debugging with symbols subjectively easier.
   * Can be serialized, though a separate struct will be needed if you want to serialize fields that are not part of the
     request body.

   **Cons**

   * You likely need separate structs for the body and parameters because callers may need to serialize URL parameters.
     This means cloning or copying memory, or taking references that may require defining lifetime parameters which
     may complicate the API for callers.

     For a data plane client library, after some internal discussion we also don't see a value in this like there may
     be for control plane (ARM).

    **Decision**

    While this may improve code reviews and possibility debugging, we can think of no strong reason to necessarily supporting
    deserialization of client library parameters. Request and response models should, no doubt, but required method parameters
    that typically correspond to required URL path or query string parameters it not something customers of our other
    Azure SDK languages have ever asked for nor have we seen use cases to warrant feature work.

    Add to that, deciding what should or shouldn't be serializable and that customers have to create a struct just to call a simple
    method like `set_string(name: &str, value: &str)` we feels makes this client library unwieldy.

[client_builder_method_builder]: sdk/client_builder_method_builder/examples/set_secret_client_builder.rs
[client_new_method_builder]: sdk/client_new_method_builder/examples/set_secret_method_builder.rs
[client_new_method_params]: sdk/client_new_method_params/examples/set_secret_params.rs
[client_new_method_params_context]: sdk/client_new_method_params_context/examples/set_secret_params_context.rs
[client_new_method_params_struct]: sdk/client_new_method_params_struct/examples/set_secret_params_struct.rs

## Conventions

* Builder fields should be private.
* Builder setters should be declared as `with_field_name(&mut self, value: impl Into<FieldType>) -> &mut Self`.
* Client fields should be private.
* ClientOptions should be public.
* Model fields should be public.
* Parameter types that do not require concrete types should use `impl Into<ParamType>` for owned values or `impl AsRef<ParamType>` if not owned.
* Private field getters should be declared as `field_name(&self) -> &field_type`.
* Private field setters should be declared as `set_field_name(&mut self, value: impl Into<FieldType>)`.
* Type construction should use `new(...)` or `with_more_details(...)`. Consider implementing `Default` as appropriate.

See <https://rust-lang.github.io/api-guidelines/naming.html> for additional naming guidelines.

## License

Licensed under the [MIT](LICENSE.txt) license.

[Azure/azure-sdk-for-rust]: https://github.com/Azure/azure-sdk-for-rust
[guidelines]: https://azure.github.io/azure-sdk/general_introduction.html
[typestate builder pattern]: https://gist.github.com/heaths/1eb608df947de5d5b47da0ee6a5a5c6d

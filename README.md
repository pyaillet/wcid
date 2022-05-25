# [ARCHIVED]

You can use `kubectl auth can-i --list` instead of this.

Example:
```shell-session
$ kubectl auth can-i --list
Resources                                       Non-Resource URLs   Resource Names   Verbs
*.*                                             []                  []               [*]
                                                [*]                 []               [*]
selfsubjectaccessreviews.authorization.k8s.io   []                  []               [create]
selfsubjectrulesreviews.authorization.k8s.io    []                  []               [create]
                                                [/api/*]            []               [get]
                                                [/api]              []               [get]
                                                [/apis/*]           []               [get]
                                                [/apis]             []               [get]
                                                [/healthz]          []               [get]
                                                [/healthz]          []               [get]
                                                [/livez]            []               [get]
                                                [/livez]            []               [get]
                                                [/openapi/*]        []               [get]
                                                [/openapi]          []               [get]
                                                [/readyz]           []               [get]
                                                [/readyz]           []               [get]
                                                [/version/]         []               [get]
                                                [/version/]         []               [get]
                                                [/version]          []               [get]
                                                [/version]          []               [get]
```

# WCID: What Can I Do?

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
![Nightly](https://github.com/pyaillet/wcid/actions/workflows/nightly.yml/badge.svg)



This project is a learning exercise to use the [Kubernetes](http://kubernetes.io/) API in [Rust](https://www.rust-lang.org/fr).

It will show you what the current user (from kubeconfig or service account) can
do with the K8s cluster.

## Usage

<img src="./resources/help.png" alt="Usage" width="70%" />

## Example

<img src="./resources/example.png" alt="Example" width="70%" />

## Note

The default settings of this tool use native-tls.
However, the current implementation of [native-tls](https://crates.io/crates/native-tls) does [not support TLS 1.3](https://github.com/sfackler/rust-native-tls/issues/140)

There is a feature to activate [rustls-tls](https://github.com/ctz/rustls) which uses TLS 1.3 and performs better.
However it curently does [not support validation of certificate presenting an IP address](https://github.com/ctz/rustls/issues/184). So be aware that using rustls-tls feature will fail when your kubernetes API server certificate presents an IP address.
If you want to use `rustls`, build the project with:

```shell
cargo build --release --no-default-features --features rustls-tls
```

## Credits

Inspired by [rakkess](https://github.com/corneliusweig/rakkess)

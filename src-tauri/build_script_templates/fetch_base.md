This is used to fetch the base library, which if cached, should speed up
subsequent builds.

```ucm
{{ for dependency in dependencies_cache }}
.> clone {dependency}
{{ endfor }}
```

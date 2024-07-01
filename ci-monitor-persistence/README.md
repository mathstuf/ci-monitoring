# ci-monitor-persistence

This crate defines traits which can serve as storage and persistence for
`ci-monitor-core` data structures. Some simple in-memory implementations are
provided for data structures. For blob storage, a simple synchronous
filesystem-backed implementation is provided.

# Demo 1 — Results

## SHA256: originals (pre-compose)

```
1389bc409c1f4c8dcde17180ff76879d6e8899c2bd12e9f180444c9f5a9b387f *format-rules.md
47197411efac4fddc82311644b9dbd9d553246c385daf7d0a1888a8cfcc0ca3a *role-statement.md
92820f6dc0acfea5bd846a465850504b888b68f867f6fa694379873428130519 *safety-fence.md
eddd586f466eb54e37454a8ca38154799e7a3da5dcc0614d6f1a2306b90aa828 *tone-discipline.md
```

## SHA256: recovered (post-decompose-full, after originals moved away)

```
1389bc409c1f4c8dcde17180ff76879d6e8899c2bd12e9f180444c9f5a9b387f *format-rules.md
47197411efac4fddc82311644b9dbd9d553246c385daf7d0a1888a8cfcc0ca3a *role-statement.md
92820f6dc0acfea5bd846a465850504b888b68f867f6fa694379873428130519 *safety-fence.md
eddd586f466eb54e37454a8ca38154799e7a3da5dcc0614d6f1a2306b90aa828 *tone-discipline.md
```

## Per-file diff verdict

| File | Result |
|---|---|
| `role-statement.md` | BYTE-IDENTICAL |
| `safety-fence.md` | BYTE-IDENTICAL |
| `tone-discipline.md` | BYTE-IDENTICAL |
| `format-rules.md` | BYTE-IDENTICAL |

## Conclusion

All 4 SHA256 hashes match between originals and recovered. The composed file (1,267 bytes) is sufficient to reconstruct every original at byte precision, with no library access needed.

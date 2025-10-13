`intrinsic.py` is a special document, that can bypass our rule
of documentation. As these intrinsics are meaningless without
understanding the global view of our whole system.
To avoid redundancy and the excessive consistency across different
files, this `intrinsic.md` can simply say:

Each intrinsic corresponds to a builder annotated by `@ir_builder` with lower-cased name
the same as the intrinsic opcode Enum.
Refer to [docs/design/lang/intrinsic.md](../../../../docs/design/lang/intrinsics.md)
for more details on the semantics of the intrinsics.
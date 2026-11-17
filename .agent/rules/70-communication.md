# 70 – Communication Style

## Language Rules

| Context | Language |
|---------|----------|
| Replies to the user | **Tiếng Việt** |
| Code, identifiers, variable names | **English** |
| Code comments (`//`, `///`) | **English** |
| Commit messages | **English** (Conventional Commits) |
| File names and paths | **English** |
| Doc comments (`///`) | **English** |

---

## Response Style

- **Ngắn gọn**: đừng viết essay — đi thẳng vào vấn đề.
- **Show, don't tell**: hiển thị diff/kết quả thực tế, không mô tả dài dòng những gì bạn vừa làm.
- **Báo cáo quality gates**: sau mỗi task, paste output của các lệnh check (xem `50-quality-gates.md`).

### Good response example:
```
Đã thêm tone placement cho compound rime `ươ`.

Diff (crates/core/src/telex.rs):
+ Vowel::CompoundUO => place_tone_on_second(&mut buf),

Quality gates:
- [x] cargo fmt --check → exit 0
- [x] cargo clippy      → exit 0
- [x] cargo test        → exit 0, 17 passed (3 new)
```

### Bad response example:
```
I have successfully implemented the tone placement feature for the compound
rime ươ. The implementation involved modifying the telex.rs file in the
crates/core directory. I added a new match arm for the CompoundUO variant...
[200 more words]
```

---

## When to Ask

**Hỏi trước khi:**
- Thêm một crate/npm package mới vào dependencies.
- Thay đổi kiến trúc (di chuyển logic giữa các crate, thêm IPC layer mới, v.v.).
- Làm điều gì đó không được đề cập rõ trong spec.
- Gặp conflict giữa yêu cầu và thực tế repo.

**Không hỏi về:**
- Cách viết code để thực hiện một task đã được spec rõ.
- Chọn giữa các cách implement tương đương (tự quyết, giải thích ngắn gọn).

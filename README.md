# SFX (Situation Framework eXchange)

[![en](https://img.shields.io/badge/lang-en-red.svg)](https://github.com/roriau0422/sfex-lang/blob/main/README-en.md)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://github.com/roriau0422/sfex-lang/actions/workflows/rust.yml/badge.svg)](https://github.com/roriau0422/sfex-lang/actions)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/roriau0422/sfex-lang)](https://github.com/roriau0422/sfex-lang/releases)

Rust дээр бичиж байгаа context-oriented програмчлалын хэл. Гол санаа нь: объектууд одоогийн нөхцөл байдал буюу situation-с хамаарч өөр өөрөөр ажиллах ёстой - яг л чи ажил дээрээ өөрөөр, гэртээ өөрөөр биеэ авч явдаг шиг.

```sfex
Story:
    Print "Hello, SFX!"
    
    Numbers is [10, 20, 30]
    Print Numbers[1]  # хариу 10 - тийм ээ индекс 1-ээс эхэлдэг
```

## SFX гэж юу вэ?

SFX бол анхны бие даасан Context-Oriented Programming хэл. Өмнөх COP implementation-ууд (ContextJ, ContextPy гэх мэт) бүгд бусад хэлнүүд дээр залгаж тавьсан extension-ууд байсан. SFX-д `Situation` болон `Switch` syntax хэлний үндсэн бүтцэд шууд орсон.

Уламжлалт OOP-ийн ёс журмаас залхаад л эхлүүлсэн юм. Яагаад объектууд зүгээр л... context-оос хамаарч өөрчлөгдөж болдоггүй юм бэ? Бодит амьдрал дээр `User` объект админ үед өөрөөр, зочин үед өөрөөр ажилладаг шүү дээ. Ихэнх хэлнүүдэд strategy pattern, dependency injection, эсвэл runtime type check хэрэгтэй болдог. SFX-д:

```sfex
Situation: AdminMode
    Adjust User:
        To GetPermissions:
            Return "admin,write,delete"

Concept: User
    To GetPermissions:
        Return "read"

Story:
    Create User Called Bob
    Print Bob.GetPermissions       # "read"
    
    Switch on AdminMode
    Print Bob.GetPermissions       # "admin,write,delete"
    
    Switch off AdminMode
    Print Bob.GetPermissions       # "read"
```

## Одоогийн байдал

**Ажиллаж байгаа:**
- Python маягийн indentation-тай Lexer/parser
- Tree-walking interpreter
- Cranelift ашигласан JIT (100 удаа дуудагдсаны дараа идэвхждэг)
- Reactive `When` observer-ууд
- Standard library: HTTP, WebSocket, TCP, JSON, CSV, XML, HTML, TOML, LLM, File I/O
- `Do in background` болон channel-тай async
- 1-ээс эхэлдэг index, arbitrary precision тоо

**Ажиллахгүй байгаа / TODO:**
- Debugger байхгүй
- LSP / editor support байхгүй
- Error message-үүд WORSE
- Баримтжуулалт дутуу
- Package manager байхгүй

## Суулгах

```bash
git clone https://github.com/roriau0422/sfex-lang.git
cd sfex-lang
cargo build --release
./target/release/sfex run your_script.sfex
```

Rust 1.75+ хэрэгтэй.

## Дизайны шийдвэрүүд

Хачин санагдаж магадгүй зарим шийдвэрүүд:

**1-ээс эхлэх index:** `List[1]` бол эхний элемент. Ингэж л хүмүүс тоолдог шүү дээ. Lua ч тэгдэг. R ч тэгдэг. MATLAB ч тэгдэг.

**Default-аар arbitrary precision:** SFX дээр `0.1 + 0.2 = 0.3`, `0.30000000000000004` биш. Хурд хэрэгтэй бол `FastNumber` хэрэглэ.

**Null байхгүй:** Хувьсагчид аюулгүй утгаар эхэлдэг (0, "", False, []). "Утга байхгүй" гэж хэрэгтэй бол `Option`-г `Some(x)` эсвэл `None`-тэй хэрэглэ.

**Grapheme-aware string:** `"👨‍👩‍👧‍👦".Length` бол 1, 7 биш. Нэг тэмдэгт учраас.

## Syntax тойм

```sfex
# Хувьсагч
Name is "Alice"
Age is 25
Items is [1, 2, 3]

# Concept (class шиг юм)
Concept: Person
    Name, Age
    
    To Greet:
        Print "Hi, I'm " + This.Name

# Control flow
If Age > 18:
    Print "Adult"
Else:
    Print "Minor"

Repeat 10 times:
    Print "Hello"

For each Item in Items:
    Print Item

# Pattern matching
When Score:
    is 100:
        Print "Perfect"
    is 90:
        Print "Great"
    Otherwise:
        Print "OK"
```

## Reactive Observer-ууд

Миний хамгийн дуртай feature. `When` block тодорхой нөхөлд автоматаар ажилладаг:

```sfex
Concept: Product
    Price, Tax, Total
    
    When Price changes:
        Set This.Tax to This.Price * 0.1
        Set This.Total to This.Price + This.Tax

Story:
    Create Product Called Phone
    Set Phone.Price to 100
    # Tax одоо 10, Total одоо 110 - автоматаар
```

Pub/sub boilerplate байхгүй. Гараар invalidate хийх хэрэггүй. Зүгээр л ажилладаг.

## Standard Library

| Модуль | Юу хийдэг |
|--------|-----------|
| HTTP | GET/POST/PUT/DELETE |
| WebSocket | Bidirectional real-time |
| TCP/UDP | Low-level socket |
| JSON/XML/HTML/CSV/TOML | Parse хийх, үүсгэх |
| Data | Формат автоматаар таниад parse хийх |
| File | Унших/бичих/stream |
| Env | Environment variable, .env support |
| System | Shell command |
| Time | Огноо/цаг |
| Math | Random, тригонометр, бөөрөнхийлөх |
| LLM | OpenAI API integration |
| Task/Channel | Concurrency primitive |

## Performance

JIT нь Cranelift хэрэглэдэг. Function 100 удаа дуудагдсаны дараа native код болж compile хийгддэг. AMD Ryzen дээрх миний benchmark:

- Энгийн арифметик loop: ~230M iteration/sec (JIT) vs ~45M (interpreted)
- Fibonacci(30): ~3M call/sec

Эдгээр тоонуудыг бүрэн итгэж болохгүй. Microbenchmark худлаа ярьдаг. Бодит performance чиний бодит кодоос хамаарна.

## Яагаад "SFX"?

**S**ituation **F**ramework e**X**change. Бас сайхан сонсогддог.

## Хамтран ажиллах

Ганцаараа хийж байгаа project, гэхдээ дараах зүйлсэд туслах хүнд баяртай талархах болно:
- Илүү сайн error message
- Test coverage
- Баримтжуулалт
- LSP implementation гоё байх байсан

Issue-г https://github.com/roriau0422/sfex-lang/issues дээр бичээрэй

## Лиценз

Apache 2.0

## Холбоо барих

Тэмүүжин - roriau@gmail.com

---

*Эрт байгаа. Эвдрэх зүйлс байх. Гэхдээ үндсэн санаа ажиллаж байгаа, идэвхтэй хөгжүүлж байна.*

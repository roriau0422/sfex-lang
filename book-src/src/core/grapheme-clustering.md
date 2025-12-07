# Grapheme Clustering

## The Emoji Problem

Quick question: How many characters is this emoji?

```
👨‍👩‍👧‍👦
```

Most programming languages say:

```python
# Python
emoji = "👨‍👩‍👧‍👦"
len(emoji)  # 7 ❌
```

```javascript
// JavaScript
let emoji = "👨‍👩‍👧‍👦";
emoji.length;  # 11 ❌
```

```java
// Java
String emoji = "👨‍👩‍👧‍👦";
emoji.length();  // 11 ❌
```

But it's **ONE emoji**! The "family" emoji is a single visual unit.

SFX gets it right:

```sfex
Story:
    Family is "👨‍👩‍👧‍👦"
    Length is Family.Length  # 1 ✓
```

## What Are Grapheme Clusters?

A **grapheme cluster** is what humans perceive as a single character:

- `a` - Simple character (1 grapheme)
- `é` - Can be one character OR `e` + combining accent (still 1 grapheme)
- `👨‍👩‍👧‍👦` - Multiple Unicode code points (but 1 grapheme)
- `🇺🇸` - Two code points (but 1 flag = 1 grapheme)

### Technical Details (Optional)

Unicode has different representations:

1. **Code points** - Individual Unicode values (U+0041, U+1F600, etc.)
2. **Code units** - UTF-8 bytes, UTF-16 words
3. **Grapheme clusters** - What humans see as characters ✓

Most languages count code points or code units. SFX counts grapheme clusters.

## Why This Matters

### 1. String Length

```sfex
Story:
    # Simple ASCII
    Name is "Alice"
    Print Name.Length  # 5 ✓

    # Emoji
    Emoji is "🎉"
    Print Emoji.Length  # 1 ✓

    # Complex emoji
    Family is "👨‍👩‍👧‍👦"
    Print Family.Length  # 1 ✓

    # Flag emoji
    Flag is "🇺🇸"
    Print Flag.Length  # 1 ✓

    # Combining diacritics
    Accented is "é"  # Can be one code point or e + ́
    Print Accented.Length  # 1 ✓ (regardless of representation)
```

Compare to other languages:

```python
# Python counts code points
"🎉".len()           # 1 (lucky - simple emoji)
"👨‍👩‍👧‍👦".len()  # 7 ❌ (complex emoji)
"🇺🇸".len()          # 2 ❌ (flag is 2 code points)
```

### 2. String Slicing

```sfex
Story:
    Text is "Hello 👋 World 🌍"

    # Get first 7 graphemes
    Slice is Text.Slice(1, 7)
    Print Slice  # "Hello 👋" ✓ (emoji counts as 1)
```

Other languages:

```python
# Python
text = "Hello 👋 World 🌍"
slice = text[0:7]  # Might cut emoji in half! 💥
```

### 3. Text Truncation

```sfex
Concept: TextTruncator
    To Truncate with Text and MaxLength:
        If Text.Length <= MaxLength:
            Return Text
        Else:
            Truncated is Text.Slice(1, MaxLength - 3)
            Return Truncated + "..."

Story:
    Create TextTruncator Called T

    Short is "Hello"
    Print T.Truncate with Short and 10  # "Hello"

    Long is "Hello 👋 World 🌍 Everyone 🎉"
    Print T.Truncate with Long and 15  # "Hello 👋 World..." ✓
    # Emoji not split!
```

### 4. Character Validation

```sfex
Story:
    # Validate username length
    Username is "Alice🎮"
    MinLength is 3
    MaxLength is 20

    If Username.Length >= MinLength and Username.Length <= MaxLength:
        Print "Valid username"  # This prints ✓
        # "Alice🎮" is 6 graphemes (A-l-i-c-e-🎮)
```

### 5. Display Width

```sfex
Concept: TextRenderer
    To PadRight with Text and Width:
        # Pad with spaces to reach width
        CurrentLength is Text.Length
        If CurrentLength >= Width:
            Return Text
        Else:
            Padding is Width - CurrentLength
            Spaces is ""
            Repeat Padding times:
                Spaces is Spaces + " "
            Return Text + Spaces

Story:
    Create TextRenderer Called Renderer

    # All aligned, even with emoji
    Print Renderer.PadRight with "Name" and 15 + "| Status"
    Print Renderer.PadRight with "Alice" and 15 + "| Active"
    Print Renderer.PadRight with "Bob 🎮" and 15 + "| Offline"
    # Output:
    # Name           | Status
    # Alice          | Active
    # Bob 🎮         | Offline
```

## Common Emoji Patterns

### Simple Emoji

```sfex
Story:
    # Single code point emoji
    Smile is "😀"     # 1 grapheme ✓
    Heart is "❤️"     # 1 grapheme ✓
    Star is "⭐"      # 1 grapheme ✓

    Total is Smile.Length + Heart.Length + Star.Length
    Print Total  # 3 ✓
```

### Skin Tone Modifiers

```sfex
Story:
    # Emoji + skin tone = 1 grapheme
    WaveLight is "👋🏻"   # 1 grapheme ✓
    WaveDark is "👋🏿"    # 1 grapheme ✓

    Print WaveLight.Length  # 1 ✓
```

### Combined Emoji (ZWJ Sequences)

```sfex
Story:
    # Zero-Width Joiner combines emoji
    Family is "👨‍👩‍👧‍👦"        # 1 grapheme ✓
    Couple is "👨‍❤️‍👨"         # 1 grapheme ✓
    FemaleFirefighter is "👩‍🚒"  # 1 grapheme ✓

    Total is Family.Length + Couple.Length + FemaleFirefighter.Length
    Print Total  # 3 ✓
```

### Flag Emoji

```sfex
Story:
    # Two regional indicator symbols = 1 flag
    US is "🇺🇸"      # 1 grapheme ✓
    UK is "🇬🇧"      # 1 grapheme ✓
    Japan is "🇯🇵"   # 1 grapheme ✓

    Flags is US + UK + Japan
    Print Flags.Length  # 3 ✓
```

## Real-World Examples

### Tweet Length Counter

```sfex
Concept: TweetValidator
    MaxLength is 280

    To IsValid with Text:
        Return Text.Length <= This.MaxLength

    To GetRemainingChars with Text:
        Return This.MaxLength - Text.Length

Story:
    Create TweetValidator Called Validator

    Tweet is "Hello world! 👋🌍🎉"

    If Validator.IsValid with Tweet:
        Remaining is Validator.GetRemainingChars with Tweet
        Print "Valid! " + Remaining + " characters remaining"
        # Counts emoji correctly!
```

### Username Validation

```sfex
Concept: UsernameValidator
    MinLength is 3
    MaxLength is 20

    To Validate with Username:
        Length is Username.Length

        If Length < This.MinLength:
            Return "Username too short (min " + This.MinLength + " characters)"
        Else If Length > This.MaxLength:
            Return "Username too long (max " + This.MaxLength + " characters)"
        Else:
            Return "Valid"

Story:
    Create UsernameValidator Called Validator

    # All valid - emoji count as 1 character each
    Print Validator.Validate with "Alice"      # Valid
    Print Validator.Validate with "Bob🎮"      # Valid
    Print Validator.Validate with "游戏玩家"    # Valid (Chinese characters)
    Print Validator.Validate with "مستخدم"    # Valid (Arabic)
```

### Text Editor

```sfex
Concept: TextEditor
    Content
    CursorPosition

    To MoveCursorRight:
        If This.CursorPosition < This.Content.Length:
            Set This.CursorPosition to This.CursorPosition + 1

    To MoveCursorLeft:
        If This.CursorPosition > 0:
            Set This.CursorPosition to This.CursorPosition - 1

    To DeleteCharacter:
        # Delete character at cursor
        Before is This.Content.Slice(1, This.CursorPosition - 1)
        After is This.Content.Slice(This.CursorPosition + 1, This.Content.Length)
        Set This.Content to Before + After
        # Deletes entire grapheme cluster, not just one byte!

Story:
    Create TextEditor Called Editor
    Set Editor.Content to "Hello 👨‍👩‍👧‍👦 World"
    Set Editor.CursorPosition to 7

    # Delete the family emoji
    Editor.DeleteCharacter
    Print Editor.Content  # "Hello World" ✓
    # Entire emoji deleted, not corrupted!
```

## Performance

Grapheme clustering is slightly slower than byte counting:

```sfex
Story:
    # For ASCII-only strings, fast
    Text is "Hello World"
    Length is Text.Length  # Fast

    # For Unicode strings, slightly slower
    Text is "Hello 👨‍👩‍👧‍👦 World 🌍"
    Length is Text.Length  # Slightly slower (but still fast!)
```

**But correctness > speed**. And with JIT compilation, the difference is minimal.

## Best Practices

### 1. Use .Length for Character Count

```sfex
# Good - grapheme count
Text is "Hello 👋"
Count is Text.Length  # 7 (H-e-l-l-o-space-👋)
```

### 2. Use .ByteSize for Storage Size

```sfex
# If you need byte count for storage/network
Text is "Hello 👋"
Bytes is Text.ByteSize  # More than 7 (UTF-8 bytes)
```

### 3. Slice by Graphemes

```sfex
# SFX slices by graphemes
Text is "Hello 👋 World"
First7 is Text.Slice(1, 7)  # "Hello 👋" (emoji not split)
```

### 4. Validate Input by Grapheme Count

```sfex
# Validate display length, not byte length
Username is "User🎮"
If Username.Length > 20:
    Print "Username too long"
```

## Comparison with Other Languages

| Language | "👨‍👩‍👧‍👦".Length | Method |
|----------|---------|--------|
| SFX | 1 ✓ | Grapheme clusters |
| Python | 7 | Code points |
| JavaScript | 11 | UTF-16 code units |
| Java | 11 | UTF-16 code units |
| Go | 25 | UTF-8 bytes |
| Rust | 25 | UTF-8 bytes (default) |
| Swift | 1 ✓ | Grapheme clusters |

SFX and Swift get it right by default!

## Summary

SFX uses grapheme clustering for strings:

✓ **Human-centric** - Counts what humans see as characters
✓ **Emoji-friendly** - 👨‍👩‍👧‍👦 is 1 character, not 7
✓ **International** - Works with all languages (Chinese, Arabic, emoji, etc.)
✓ **Safe slicing** - Won't split multi-byte characters
✓ **Correct validation** - Username/tweet length validation works correctly

When dealing with text in the 21st century, grapheme clustering is essential.

---

Next: [Basic Syntax](../syntax/basics.md) - Learn SFX syntax fundamentals

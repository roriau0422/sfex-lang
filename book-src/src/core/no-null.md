# No Null Pointers

## The Billion-Dollar Mistake

In 2009, Tony Hoare, inventor of null references, called it his "billion-dollar mistake":

> "I call it my billion-dollar mistake... null references have led to innumerable errors, vulnerabilities, and system crashes, which have probably caused a billion dollars of pain and damage."

```java
// Java - the typical null nightmare
String name = null;
int length = name.length();  // NullPointerException! 💥
```

```javascript
// JavaScript
let user = null;
console.log(user.email);  // TypeError: Cannot read property 'email' of null 💥
```

```python
# Python
person = None
age = person.age  # AttributeError: 'NoneType' object has no attribute 'age' 💥
```

These crashes happen because:
1. Variables can be null
2. You forget to check for null
3. Program crashes at runtime

## The SFX Solution

SFX eliminates null pointers with two approaches:

### 1. Safe Defaults

**Every type has a sensible default value:**

```sfex
Concept: Person
    Name    # Defaults to "" (empty string)
    Age     # Defaults to 0
    Active  # Defaults to False
    Tags    # Defaults to [] (empty list)
    Meta    # Defaults to {} (empty map)

Story:
    Create Person Called Bob
    # No need to initialize - safe defaults!

    Print Bob.Name    # "" (empty, not null)
    Print Bob.Age     # 0
    Print Bob.Active  # False
    Print Bob.Tags    # []

    # Safe to use immediately
    Length is Bob.Name.Length  # 0 (works! no crash!)
```

### 2. Option Type

**For values that might be absent, use Option:**

```sfex
# Some(value) or None
Result is Some(42)
NoResult is None

# Check if present
If Result.IsSome:
    Print "Has value!"

If NoResult.IsNone:
    Print "No value!"

# Extract value safely
If Result.IsSome:
    Value is Result.Unwrap()
    Print Value  # 42

# Provide fallback
Default is NoResult.UnwrapOr(0)
Print Default  # 0 (because NoResult is None)
```

## Why This is Better

### No Null Pointer Crashes

```sfex
# SFX - always safe
Concept: User
    Email

Story:
    Create User Called Guest

    # Email is "" (empty string), not null
    Length is Guest.Email.Length  # 0 ✓ (no crash!)
    If Guest.Email = "":
        Print "No email provided"
```

Compare to:
```java
// Java - potential crash
User guest = new User();
int length = guest.email.length();  // NullPointerException if email is null!
```

### Explicit Optional Values

```sfex
Concept: SearchResult
    Result  # Option type

    To Find with Query:
        # Simulate search
        If Query = "found":
            Set This.Result to Some("Item found!")
        Else:
            Set This.Result to None

Story:
    Create SearchResult Called Search

    Search.Find with "found"
    If Search.Result.IsSome:
        Print Search.Result.Unwrap()  # "Item found!"

    Search.Find with "missing"
    If Search.Result.IsNone:
        Print "Nothing found"  # This prints
```

### Safe Method Chaining

```sfex
Concept: Config
    Database  # Map with database config

Story:
    Create Config Called Settings
    Set Settings.Database to {}  # Empty map (not null!)

    # Safe to access nested properties
    Host is Settings.Database["host"]  # Returns "" if not set
    Port is Settings.Database["port"]  # Returns 0 if not set

    # No crashes!
```

## Default Values by Type

| Type | Default Value | Example |
|------|--------------|---------|
| Number | 0 | `Age` defaults to `0` |
| FastNumber | 0.0 | `Speed` defaults to `0.0` |
| String | "" | `Name` defaults to `""` |
| Boolean | False | `Active` defaults to `False` |
| List | [] | `Items` defaults to `[]` |
| Map | {} | `Config` defaults to `{}` |
| Option | None | `Result` defaults to `None` |

## Option Type API

### Creating Options

```sfex
Story:
    HasValue is Some(42)
    NoValue is None

    # From expressions
    Result is If Score > 50 then Some(Score) else None
```

### Checking for Values

```sfex
Story:
    Result is Some(100)

    # Check if has value
    If Result.IsSome:
        Print "Has value"

    If Result.IsNone:
        Print "No value"
```

### Extracting Values

```sfex
Story:
    Result is Some(42)

    # Unwrap (crashes if None!)
    Value is Result.Unwrap()
    Print Value  # 42

    # Unwrap with default (safe)
    Value is Result.UnwrapOr(0)
    Print Value  # 42 (or 0 if None)

    NoResult is None
    Value is NoResult.UnwrapOr(99)
    Print Value  # 99 (fallback used)
```

## Real-World Examples

### Database Query

```sfex
Concept: Database
    To FindUser with Id:
        # Returns Option - user might not exist
        If Id = 1:
            Return Some({ name: "Alice", age: 30 })
        Else:
            Return None

Story:
    Create Database Called DB

    Result is DB.FindUser with 1
    If Result.IsSome:
        User is Result.Unwrap()
        Print "Found: " + User["name"]
    Else:
        Print "User not found"

    Result is DB.FindUser with 999
    If Result.IsNone:
        Print "User not found"  # This prints
```

### Configuration

```sfex
Concept: AppConfig
    Settings  # Map

    To GetSetting with Key:
        # Return Option - setting might not exist
        If This.Settings.HasKey(Key):
            Return Some(This.Settings[Key])
        Else:
            Return None

    To GetSettingWithDefault with Key and Default:
        Result is This.GetSetting with Key
        Return Result.UnwrapOr(Default)

Story:
    Create AppConfig Called Config
    Set Config.Settings to { timeout: 30, retries: 3 }

    # Get with Option
    Timeout is Config.GetSetting with "timeout"
    If Timeout.IsSome:
        Print "Timeout: " + Timeout.Unwrap()

    # Get with default
    MaxConnections is Config.GetSettingWithDefault with "max_connections" and 100
    Print "Max connections: " + MaxConnections  # 100 (default used)
```

### Parsing

```sfex
Concept: Parser
    To ParseInt with Text:
        # Try to parse, return Option
        Try:
            Number is Int.Parse(Text)
            Return Some(Number)
        Catch Error:
            Return None

Story:
    Create Parser Called P

    Result is P.ParseInt with "42"
    If Result.IsSome:
        Num is Result.Unwrap()
        Print "Parsed: " + Num  # 42

    Result is P.ParseInt with "not a number"
    If Result.IsNone:
        Print "Parse failed"  # This prints
```

### Caching

```sfex
Concept: Cache
    Data  # Map

    To Get with Key:
        If This.Data.HasKey(Key):
            Return Some(This.Data[Key])
        Else:
            Return None

    To GetOrCompute with Key and ComputeFunc:
        Cached is This.Get with Key
        If Cached.IsSome:
            Return Cached.Unwrap()
        Else:
            Value is ComputeFunc()
            This.Data[Key] is Value
            Return Value

Story:
    Create Cache Called MyCache
    Set MyCache.Data to {}

    # Cache miss - compute
    Value is MyCache.GetOrCompute with "expensive" and ExpensiveCalculation
    Print Value

    # Cache hit - return cached
    Value is MyCache.GetOrCompute with "expensive" and ExpensiveCalculation
    Print Value  # Same value, no recomputation
```

## Best Practices

### 1. Use Safe Defaults for Always-Present Data

```sfex
Concept: Person
    Name     # Always has a name (even if "")
    Age      # Always has an age (even if 0)
    Email    # Always has an email (even if "")

    # Safe defaults, no null checks needed
```

### 2. Use Option for Maybe-Present Data

```sfex
Concept: User
    Username
    ProfilePicture  # Option - might not have one

    To GetProfilePicture:
        # Return Option
        If This.ProfilePicture.IsSome:
            Return This.ProfilePicture
        Else:
            Return None
```

### 3. Provide Fallbacks with UnwrapOr

```sfex
Story:
    Config is LoadConfig()
    Timeout is Config.GetSetting with "timeout"

    # Use fallback if not set
    ActualTimeout is Timeout.UnwrapOr(30)
```

### 4. Check Before Unwrap

```sfex
# Good - safe
If Result.IsSome:
    Value is Result.Unwrap()

# Risky - crashes if None
Value is Result.Unwrap()
```

### 5. Use Option for Return Values That Can Fail

```sfex
Concept: Finder
    To Find with Query:
        # Might not find anything
        If Found:
            Return Some(Result)
        Else:
            Return None  # Explicit: no result
```

## Comparison with Other Languages

| Language | Null Handling |
|----------|--------------|
| Java | `null` everywhere, can crash anytime |
| JavaScript | `null` and `undefined`, both cause crashes |
| Python | `None`, causes crashes |
| Rust | `Option<T>`, must handle (like SFX) |
| Kotlin | Nullable types `T?`, must check |
| SFX | Safe defaults + Option type ✓ |

SFX combines the best of both worlds:
- **Safe defaults** for simplicity (beginners)
- **Option type** for explicit optionality (advanced)

## Summary

SFX eliminates null pointer crashes through:

✓ **Safe defaults** - Every type has a sensible default (0, "", False, [], {})
✓ **Option type** - Explicit handling of maybe-present values
✓ **No null crashes** - Can't access null, because there is no null
✓ **Clearer code** - Intent is explicit (Some vs None)
✓ **Beginner-friendly** - Safe by default, no crashes

The result? **Fewer bugs, clearer intent, safer code.**

---

Next: [Grapheme Clustering](./grapheme-clustering.md) - Unicode done right

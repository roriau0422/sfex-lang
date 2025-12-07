# Data Types

SFX has a rich type system designed to be both beginner-friendly and powerful. All types are automatically inferred from values.

## Overview

| Type | Description | Example |
|------|-------------|---------|
| Number | Arbitrary precision decimal | `42`, `3.14` |
| FastNumber | IEEE 754 float (f64) | `FastNumber(3.14)` |
| String | Grapheme-aware text | `"Hello"`, `"""Multi-line"""` |
| Boolean | True or False | `True`, `False` |
| List | 1-indexed dynamic array | `[1, 2, 3]` |
| Map | Key-value dictionary | `{ name: "Alice" }` |
| Vector | f32 array for AI/embeddings | `Vector([1.0, 2.0])` |
| Option | Some(value) or None | `Some(42)`, `None` |
| WeakRef | Weak reference | `WeakRef(object)` |
| TaskHandle | Concurrent task handle | From `Do in background` |
| Error | Error object | From `Try/Catch` |

## Number (Default)

**Arbitrary precision decimal** - the default numeric type.

```sfex
Story:
    # Mathematical honesty
    Result is 0.1 + 0.2
    Print Result  # 0.3 ✓

    # Large integers
    BigInt is 123456789123456789123456789
    Print BigInt

    # Decimals
    Price is 19.99
    Tax is Price * 0.08
    Total is Price + Tax
    Print Total  # 21.5892 (exact)
```

**When to use:** Financial calculations, counters, user-facing math, learning/teaching.

**Properties:**
- `.Sign` - Returns -1, 0, or 1

See [Numbers](./types/numbers.md) for details.

## FastNumber

**IEEE 754 f64 float** - for performance-critical code.

```sfex
Story:
    # Explicit creation
    Speed is FastNumber(299792458.0)
    Time is FastNumber(2.5)
    Distance is Speed * Time

    # About 10x faster than Number
    # But float errors return:
    Test is FastNumber(0.1) + FastNumber(0.2)
    Print Test  # 0.30000000000000004
```

**When to use:** Physics simulations, graphics, games, high-frequency calculations.

**Mixed operations:**
```sfex
Story:
    A is 10              # Number
    B is FastNumber(20.0)  # FastNumber
    C is A + B           # Result is FastNumber (loses precision)
```

See [FastNumber](./types/fastnumber.md) for details.

## String

**Grapheme-aware Unicode text** - emoji counted correctly.

```sfex
Story:
    # Simple string
    Name is "Alice"

    # With escape sequences
    Message is "Line 1\nLine 2\tTabbed"

    # Multiline (triple quotes)
    Text is """This is
a multiline
string"""

    # Concatenation
    Greeting is "Hello, " + Name
    Print Greeting  # "Hello, Alice"

    # Emoji counted correctly
    Emoji is "👨‍👩‍👧‍👦"
    Print Emoji.Length  # 1 ✓ (one grapheme cluster)
```

**Properties:**
- `.Length` - Number of graphemes (human-perceived characters)
- `.ByteSize` - Number of UTF-8 bytes

**Methods:**
- `.ToUpper` - Convert to uppercase
- `.ToLower` - Convert to lowercase
- `.Trim` - Remove leading/trailing whitespace
- `.Contains` - Check if substring exists
- `.Slice(start, end)` - Extract substring (1-based)

See [Strings](./types/strings.md) for details.

## Boolean

**True or False** - for logical operations.

```sfex
Story:
    IsActive is True
    IsDeleted is False

    # From comparisons
    IsAdult is Age >= 18
    HasAccess is Role = "admin"

    # Logical operations
    CanEdit is IsActive and HasAccess
    ShouldHide is IsDeleted or not IsActive
```

**Values:** `True`, `False` (case-sensitive)

See [Boolean](./types/boolean.md) for details.

## List

**1-indexed dynamic array** - the first element is at index 1.

```sfex
Story:
    # Create empty list
    Items is []

    # Create with values
    Numbers is [1, 2, 3, 4, 5]

    # Multiline
    Colors is [
        "red",
        "green",
        "blue"
    ]

    # Access (1-based!)
    First is Numbers[1]   # 1
    Third is Numbers[3]   # 3

    # Modify
    Numbers[2] is 20
    Print Numbers  # [1, 20, 3, 4, 5]

    # Add elements
    Numbers.Add(6)
    Print Numbers  # [1, 20, 3, 4, 5, 6]

    # Length
    Count is Numbers.Length
    Last is Numbers[Count]
```

**Properties:**
- `.Length` or `.Size` - Number of elements

**Methods:**
- `.Add(item)` - Append element
- `.Remove(index)` - Remove at index (1-based)
- `.Contains(item)` - Check if element exists
- `.Slice(start, end)` - Extract sublist (1-based, inclusive)
- `.Sort()` - Sort in ascending order
- `.Sort("desc")` - Sort in descending order
- `.Reverse()` - Reverse order
- `.Clear()` - Remove all elements

See [Lists](./types/lists.md) for details.

## Map

**Key-value dictionary** - associative array.

```sfex
Story:
    # Create empty map
    Config is {}

    # Create with values
    User is { name: "Alice", age: 25, email: "alice@example.com" }

    # Multiline
    Settings is {
        host: "localhost",
        port: 8080,
        debug: True
    }

    # Access
    Name is User["name"]
    Age is User["age"]

    # Modify
    User["age"] is 26

    # Add new key
    User["role"] is "admin"

    # Check if key exists
    If User.HasKey("email"):
        Print "Email: " + User["email"]
```

**Properties:**
- `.Size` or `.Length` - Number of key-value pairs

**Methods:**
- `.HasKey(key)` - Check if key exists
- `.Keys()` - Get list of all keys
- `.Values()` - Get list of all values
- `.Remove(key)` - Remove key-value pair
- `.Clear()` - Remove all entries

See [Maps](./types/maps.md) for details.

## Vector

**f32 array** - for AI embeddings and numeric arrays.

```sfex
Story:
    # Create vector
    Embedding is Vector([0.1, 0.2, 0.3, 0.4, 0.5])

    # For machine learning / AI
    UserEmbedding is Vector([/* floats */])
```

**Use case:** Storing embeddings from LLM API calls, neural network weights.

## Option

**Some(value) or None** - explicit optionality, no null pointers.

```sfex
Story:
    # Create options
    HasValue is Some(42)
    NoValue is None

    # Check
    If HasValue.IsSome:
        Print "Has value!"

    If NoValue.IsNone:
        Print "No value!"

    # Extract value
    If HasValue.IsSome:
        Value is HasValue.Unwrap()
        Print Value  # 42

    # Unwrap with default
    Value1 is HasValue.UnwrapOr(0)  # 42
    Value2 is NoValue.UnwrapOr(0)   # 0 (default)
```

**Properties:**
- `.IsSome` - Returns True if has value
- `.IsNone` - Returns True if no value

**Methods:**
- `.Unwrap()` - Extract value (crashes if None!)
- `.UnwrapOr(default)` - Extract value or return default

See [Option Types](./types/option.md) for details.

## WeakRef

**Weak reference** - reference that doesn't prevent garbage collection.

```sfex
Story:
    # Create strong reference
    Items is [1, 2, 3]

    # Create weak reference
    WeakItems is WeakRef(Items)

    # Check if still valid
    If WeakItems.IsValid:
        Print "Reference still alive"
        # Restore is WeakItems.Get()  # Not yet implemented
    Else:
        Print "Reference was collected"
```

**Properties:**
- `.IsValid` - Returns True if reference still alive

**Use case:** Caching, avoiding circular references.

See [Weak References](./types/weakref.md) for details.

## TaskHandle

**Concurrent task handle** - returned by `Do in background`.

```sfex
Story:
    # Spawn background task
    Task is Do in background:
        Result is ExpensiveCalculation()
        Return Result

    # Do other work...
    Print "Working..."

    # Wait for result
    Value is Task.Await()
    Print "Result: " + Value
```

**Methods:**
- `.Await()` - Wait for task to complete and get result

See [Concurrency](../concurrency/tasks.md) for details.

## Error

**Error object** - from Try/Catch blocks.

```sfex
Story:
    Try:
        Result is RiskyOperation()
    Catch Error:
        Print "Error: " + Error.Message
        Print "Type: " + Error.Type
```

**Properties:**
- `.Message` - Error message string
- `.Type` - Error type/category

See [Error Handling](../error-handling/errors.md) for details.

## Type Conversion

### Automatic Conversions

```sfex
Story:
    # Number to String (in concatenation)
    Age is 25
    Message is "Age: " + Age  # "Age: 25"

    # Boolean to String
    IsActive is True
    Status is "Active: " + IsActive  # "Active: True"
```

### Explicit Conversions

```sfex
Story:
    # String to Number
    Text is "42"
    Num is Int.Parse(Text)  # Use with Try/Catch!

    # FastNumber to Number
    Fast is FastNumber(3.14)
    Regular is Number(Fast)
```

## Type Checking

SFX doesn't have explicit type checking operators, but you can check values:

```sfex
Story:
    Value is 42

    # Check for specific values
    If Value = 42:
        Print "It's 42!"

    # Check empty list
    Items is []
    If Items.Length = 0:
        Print "Empty list"

    # Check empty string
    Name is ""
    If Name = "":
        Print "Empty string"
```

## Default Values (Concept Fields)

When declaring fields in Concepts without initialization:

```sfex
Concept: Example
    NumberField    # 0
    StringField    # ""
    BoolField      # False
    ListField      # []
    MapField       # {}
    OptionField    # None

Story:
    Create Example Called Ex
    Print Ex.NumberField  # 0
    Print Ex.BoolField    # False
    Print Ex.ListField.Length  # 0
```

## Nested Types

### List of Lists

```sfex
Story:
    Grid is [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ]

    # Access: row 2, column 3
    Value is Grid[2][3]  # 6
```

### List of Maps

```sfex
Story:
    Users is [
        { name: "Alice", age: 25 },
        { name: "Bob", age: 30 },
        { name: "Charlie", age: 35 }
    ]

    FirstUser is Users[1]
    Name is FirstUser["name"]  # "Alice"
```

### Map with Lists

```sfex
Story:
    Data is {
        users: ["Alice", "Bob"],
        scores: [95, 87],
        active: True
    }

    Users is Data["users"]
    FirstUser is Users[1]  # "Alice"
```

## Best Practices

### 1. Use Number by Default

```sfex
# Good - exact math
Price is 19.99
Tax is Price * 0.08
```

### 2. Use FastNumber for Performance

```sfex
# Physics/graphics - performance matters
Concept: Particle
    X, Y, VX, VY

    To Update with DT:
        FDT is FastNumber(DT)
        Set This.X to This.X + (This.VX * FDT)
```

### 3. Use Option for Maybe-Present Values

```sfex
# Good - explicit optionality
Concept: User
    ProfilePicture  # Option type

# Bad - using empty string to mean "no value"
ProfilePicture is ""  # Confusing
```

### 4. Keep Lists Homogeneous

```sfex
# Good - all same type
Numbers is [1, 2, 3, 4, 5]
Names is ["Alice", "Bob", "Charlie"]

# Bad - mixed types
Mixed is [1, "two", True]  # Confusing
```

### 5. Use Descriptive Map Keys

```sfex
# Good
User is { name: "Alice", email: "alice@example.com", age: 25 }

# Less clear
User is { n: "Alice", e: "alice@example.com", a: 25 }
```

## Summary

- **Number** - Default, arbitrary precision, mathematically honest
- **FastNumber** - Performance, IEEE 754 float
- **String** - Grapheme-aware Unicode, emoji = 1 character
- **Boolean** - True/False
- **List** - 1-based dynamic array
- **Map** - Key-value dictionary
- **Option** - Explicit optionality (no null!)
- **WeakRef** - Weak references for caching
- **Vector** - f32 arrays for AI/ML

---

Next: [Operators](./operators.md) - Arithmetic, comparison, and logical operators

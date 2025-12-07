# Variables and Assignment

Variables in SFX store data that can be used and modified throughout your program.

## Assignment with `is`

Use the `is` keyword to assign values to variables:

```sfex
Story:
    Name is "Alice"
    Age is 25
    Score is 95.5
    IsActive is True
```

## Naming Rules

Variable names in SFX:

1. **Use PascalCase** - Start with uppercase letter
2. **Can contain letters, numbers, and underscores**
3. **Case-sensitive** - `Name` and `name` are different variables
4. **Cannot start with a number**
5. **Cannot be a reserved keyword**

### Valid Names

```sfex
Story:
    UserName is "Alice"
    User2 is "Bob"
    FirstName is "Charlie"
    Total_Score is 100
    IsActive is True
    Age25 is 25
```

### Invalid Names

```sfex
Story:
    # Error - starts with number
    2User is "Invalid"

    # Error - reserved keyword
    If is "Invalid"

    # Error - starts with lowercase (convention, not enforced)
    userName is "Alice"  # Works but violates convention
```

## Variable Scope

Variables have different scopes depending on where they're declared:

### Story-Level Variables (Global)

```sfex
Story:
    GlobalVar is 100  # Visible throughout Story

    If True:
        Print GlobalVar  # Can access here
        LocalVar is 50

    # LocalVar not accessible here
```

### Method-Level Variables (Local)

```sfex
Concept: Calculator
    To Calculate:
        Result is 10 + 20  # Local to this method
        Return Result

Story:
    Create Calculator Called Calc
    Answer is Calc.Calculate
    Print Answer  # 30

    # Cannot access 'Result' here - it's local to Calculate method
```

### Block-Level Variables

```sfex
Story:
    If True:
        BlockVar is "Inside If"
        Print BlockVar  # Works

    # BlockVar not accessible here

    Repeat 3 times:
        LoopVar is "Inside Loop"
        Print LoopVar  # Works

    # LoopVar not accessible here
```

## Reassignment

Variables can be reassigned with the `is` keyword:

```sfex
Story:
    Counter is 0
    Print Counter  # 0

    Counter is 10
    Print Counter  # 10

    Counter is Counter + 5
    Print Counter  # 15
```

## Multiple Assignments

You can assign multiple variables in sequence:

```sfex
Story:
    X is 10
    Y is 20
    Z is 30

    # All separate statements
```

For object fields, you can use comma-separated `Set` statements:

```sfex
Concept: Point
    X, Y, Z

Story:
    Create Point Called P
    Set P.X to 1, Set P.Y to 2, Set P.Z to 3
```

## Type Inference

SFX automatically determines the type of a variable from its value:

```sfex
Story:
    # Number (BigDecimal)
    Age is 25

    # String
    Name is "Alice"

    # Boolean
    Active is True

    # List
    Items is [1, 2, 3]

    # Map
    User is { name: "Bob", age: 30 }

    # FastNumber (explicit)
    Speed is FastNumber(299792458.0)
```

## Default Values

When fields are declared in Concepts without initialization, they get safe defaults:

```sfex
Concept: Person
    Name    # Defaults to ""
    Age     # Defaults to 0
    Active  # Defaults to False
    Tags    # Defaults to []
    Meta    # Defaults to {}

Story:
    Create Person Called User
    Print User.Name   # ""
    Print User.Age    # 0
    Print User.Active # False
```

## Constants

SFX doesn't have a special `const` keyword, but by convention, use all-caps names for values that shouldn't change:

```sfex
Story:
    MAX_USERS is 100
    API_KEY is "secret"
    DEFAULT_TIMEOUT is 30

    # Convention: don't reassign these
```

## Variable Shadowing

Variables in inner scopes can shadow outer variables:

```sfex
Story:
    X is 10
    Print X  # 10

    If True:
        X is 20  # Shadows outer X
        Print X  # 20

    Print X  # Still 10 (outer X unchanged)
```

## Working with Different Types

### Numbers

```sfex
Story:
    # Integer-like
    Count is 42

    # Decimal
    Price is 19.99

    # Negative
    Debt is -50

    # Large numbers (arbitrary precision)
    BigNum is 123456789123456789123456789
```

### Strings

```sfex
Story:
    # Simple string
    Name is "Alice"

    # With escape sequences
    Message is "Line 1\nLine 2"

    # Multiline (triple quotes)
    Text is """This is
a multiline
string"""

    # Concatenation
    Greeting is "Hello, " + Name
```

### Booleans

```sfex
Story:
    IsActive is True
    IsDeleted is False

    # From comparisons
    IsAdult is Age >= 18
    HasAccess is Role = "admin"
```

### Lists

```sfex
Story:
    # Empty list
    Items is []

    # With values
    Numbers is [1, 2, 3, 4, 5]

    # Mixed types (not recommended)
    Mixed is [1, "two", True]

    # Multiline
    Colors is [
        "red",
        "green",
        "blue"
    ]
```

### Maps

```sfex
Story:
    # Empty map
    Config is {}

    # With values
    User is { name: "Alice", age: 25 }

    # Multiline
    Settings is {
        host: "localhost",
        port: 8080,
        debug: True
    }
```

## Common Patterns

### Swap Variables

```sfex
Story:
    A is 10
    B is 20

    # Swap using temporary variable
    Temp is A
    A is B
    B is Temp

    Print A  # 20
    Print B  # 10
```

### Accumulator Pattern

```sfex
Story:
    Sum is 0
    Numbers is [1, 2, 3, 4, 5]

    For each Num in Numbers:
        Sum is Sum + Num

    Print Sum  # 15
```

### Counter Pattern

```sfex
Story:
    Count is 0

    Repeat 10 times:
        Count is Count + 1

    Print Count  # 10
```

### Conditional Assignment

```sfex
Story:
    Score is 85

    Grade is ""
    If Score >= 90:
        Grade is "A"
    Else If Score >= 80:
        Grade is "B"
    Else:
        Grade is "C"

    Print Grade  # "B"
```

## Best Practices

### 1. Use Descriptive Names

```sfex
# Good
TotalPrice is 99.99
UserEmail is "alice@example.com"
MaxRetries is 3

# Bad
TP is 99.99
UE is "alice@example.com"
MR is 3
```

### 2. Initialize Before Use

```sfex
# Good
Counter is 0
Counter is Counter + 1

# Bad
Counter is Counter + 1  # Error if Counter doesn't exist
```

### 3. Keep Scope Minimal

```sfex
# Good - declare where needed
If Condition:
    TempResult is Calculate()
    Print TempResult

# Less good - unnecessarily wide scope
TempResult is 0
If Condition:
    TempResult is Calculate()
    Print TempResult
```

### 4. Use Meaningful Initial Values

```sfex
# Good - clear intent
UserCount is 0
IsLoggedIn is False
ErrorMessage is ""

# Less clear
UserCount is -1  # Why -1?
IsLoggedIn is 0  # Use Boolean!
```

### 5. Don't Reuse Variables for Different Purposes

```sfex
# Bad
Temp is Calculate1()
Print Temp
Temp is Calculate2()  # Confusing - different meaning
Print Temp

# Good
Result1 is Calculate1()
Print Result1
Result2 is Calculate2()
Print Result2
```

## Examples

### Configuration

```sfex
Story:
    # Application settings
    AppName is "My App"
    Version is "1.0.0"
    Debug is True

    # Database config
    DbHost is "localhost"
    DbPort is 5432
    DbName is "myapp_db"

    # API settings
    ApiTimeout is 30
    MaxRetries is 3
```

### User Input Processing

```sfex
Story:
    # Simulated user input
    UserInput is "42"

    # Parse to number
    InputNumber is 0  # Default
    Try:
        InputNumber is Int.Parse(UserInput)
    Catch Error:
        Print "Invalid input, using default"

    Print "Value: " + InputNumber
```

### Data Transformation

```sfex
Story:
    # Original data
    FirstName is "alice"
    LastName is "smith"

    # Transform
    FullName is FirstName.ToUpper + " " + LastName.ToUpper
    Print FullName  # "ALICE SMITH"

    # Calculate derived values
    Price is 100
    TaxRate is 0.08
    Tax is Price * TaxRate
    Total is Price + Tax
    Print "Total: $" + Total
```

## Summary

- Use `is` for assignment: `Name is "Alice"`
- Variables are case-sensitive: `Name` ≠ `name`
- PascalCase naming convention: `UserName`, `TotalScore`
- Type inference: SFX determines types automatically
- Block scope: Variables live within their declaration block
- Safe defaults: Concept fields default to 0, "", False, [], {}

---

Next: [Data Types](./data-types.md) - Learn about all SFX data types

# Operators

SFX provides operators for arithmetic, comparison, logical operations, and more.

## Arithmetic Operators

Perform mathematical calculations:

| Operator | Operation | Example | Result |
|----------|-----------|---------|--------|
| `+` | Addition | `10 + 5` | `15` |
| `-` | Subtraction | `10 - 5` | `5` |
| `*` | Multiplication | `10 * 5` | `50` |
| `/` | Division | `10 / 5` | `2` |
| `%` | Modulo (remainder) | `10 % 3` | `1` |

```sfex
Story:
    A is 10
    B is 3

    Sum is A + B      # 13
    Diff is A - B     # 7
    Product is A * B  # 30
    Quotient is A / B # 3.333...
    Remainder is A % B  # 1

    Print "Sum: " + Sum
    Print "Product: " + Product
    Print "Remainder: " + Remainder
```

### Mathematical Honesty

Remember, SFX uses arbitrary precision by default:

```sfex
Story:
    Result is 0.1 + 0.2
    Print Result  # 0.3 ✓ (not 0.30000000000000004)

    Price is 19.99
    Tax is Price * 0.08
    Total is Price + Tax
    Print Total  # 21.5892 (exact)
```

### Unary Minus

```sfex
Story:
    X is 10
    Y is -X   # -10
    Z is -Y   # 10
```

## Comparison Operators

Compare values and return Boolean results:

| Operator | Operation | Example | Result |
|----------|-----------|---------|--------|
| `=` | Equal to | `5 = 5` | `True` |
| `<>` | Not equal to | `5 <> 3` | `True` |
| `<` | Less than | `3 < 5` | `True` |
| `>` | Greater than | `5 > 3` | `True` |
| `<=` | Less than or equal | `5 <= 5` | `True` |
| `>=` | Greater than or equal | `5 >= 3` | `True` |

```sfex
Story:
    Age is 25
    MinAge is 18
    MaxAge is 65

    IsAdult is Age >= MinAge     # True
    IsSenior is Age >= MaxAge    # False
    IsEqual is Age = 25          # True
    IsNotEqual is Age <> 30      # True
```

### String Comparison

```sfex
Story:
    Name1 is "Alice"
    Name2 is "Bob"
    Name3 is "Alice"

    Same is Name1 = Name3        # True
    Different is Name1 <> Name2  # True

    # Lexicographic comparison
    Before is Name1 < Name2      # True ("Alice" < "Bob")
```

### Boolean Comparison

```sfex
Story:
    Active is True
    Deleted is False

    Same is Active = True        # True
    Different is Active <> Deleted  # True
```

## Logical Operators

Combine Boolean expressions:

| Operator | Operation | Example | Result |
|----------|-----------|---------|--------|
| `and` | Logical AND | `True and False` | `False` |
| `or` | Logical OR | `True or False` | `True` |
| `not` | Logical NOT | `not True` | `False` |

```sfex
Story:
    Age is 25
    HasLicense is True
    HasCar is False

    # AND - both must be true
    CanDrive is Age >= 18 and HasLicense  # True

    # OR - at least one must be true
    HasTransport is HasCar or HasBike  # Depends on HasBike

    # NOT - inverts boolean
    IsUnderAge is not (Age >= 18)  # False

    # Combined
    CanRent is Age >= 21 and HasLicense and not HasCar
```

### Truth Tables

**AND:**
| A | B | A and B |
|---|---|---------|
| True | True | True |
| True | False | False |
| False | True | False |
| False | False | False |

**OR:**
| A | B | A or B |
|---|---|--------|
| True | True | True |
| True | False | True |
| False | True | True |
| False | False | False |

**NOT:**
| A | not A |
|---|-------|
| True | False |
| False | True |

### Short-Circuit Evaluation

SFX uses short-circuit evaluation for `and` and `or`:

```sfex
Story:
    # AND - stops if first is False
    Result is False and ExpensiveCheck()  # ExpensiveCheck() not called

    # OR - stops if first is True
    Result is True or ExpensiveCheck()  # ExpensiveCheck() not called
```

## String Operators

### Concatenation (`+`)

```sfex
Story:
    FirstName is "Alice"
    LastName is "Smith"
    FullName is FirstName + " " + LastName
    Print FullName  # "Alice Smith"

    # Auto-conversion with numbers
    Age is 25
    Message is "Age: " + Age
    Print Message  # "Age: 25"
```

### Contains

```sfex
Story:
    Text is "Hello, World!"

    If Text contains "World":
        Print "Found!"

    If Text contains "Goodbye":
        Print "This won't print"
```

## List Operators

### Contains

```sfex
Story:
    Numbers is [1, 2, 3, 4, 5]

    If Numbers contains 3:
        Print "3 is in the list"

    If not (Numbers contains 10):
        Print "10 is not in the list"
```

## Operator Precedence

Operators are evaluated in this order (highest to lowest):

1. **Parentheses** - `()`
2. **Unary minus** - `-X`
3. **Multiplication, Division, Modulo** - `*`, `/`, `%`
4. **Addition, Subtraction** - `+`, `-`
5. **Comparison** - `<`, `>`, `<=`, `>=`, `=`, `<>`
6. **Logical NOT** - `not`
7. **Logical AND** - `and`
8. **Logical OR** - `or`

### Examples

```sfex
Story:
    # Multiplication before addition
    Result is 2 + 3 * 4
    Print Result  # 14 (not 20)

    # Use parentheses to override
    Result is (2 + 3) * 4
    Print Result  # 20

    # Comparison before logical
    Result is 5 > 3 and 10 < 20
    Print Result  # True

    # Complex expression
    X is 10
    Y is 5
    Z is 3
    Result is X + Y * Z > 20 and X / Y = 2
    # (10 + (5 * 3)) > 20 and (10 / 5) = 2
    # (10 + 15) > 20 and 2 = 2
    # 25 > 20 and True
    # True and True
    # True
    Print Result  # True
```

## Best Practices

### 1. Use Parentheses for Clarity

```sfex
# Less clear
Result is A + B * C / D

# Better
Result is A + ((B * C) / D)
```

### 2. Compare with Same Types

```sfex
# Good
Age is 25
IsAdult is Age >= 18  # Number to Number

# Potentially confusing
IsAdult is Age >= "18"  # Number to String - avoid!
```

### 3. Use Descriptive Names for Boolean Expressions

```sfex
# Good
IsEligible is Age >= 18 and HasLicense
CanProceed is IsAuthenticated and not IsBlocked

# Less clear
Check is Age >= 18 and HasLicense
Flag is IsAuthenticated and not IsBlocked
```

### 4. Break Complex Expressions

```sfex
# Complex - hard to read
If Age >= 18 and HasLicense and not HasCar and Income > 30000 and Score > 700:
    ApproveRental()

# Better - broken down
IsAdult is Age >= 18
HasRequirements is HasLicense and not HasCar
IsFinanciallyStable is Income > 30000 and Score > 700
IsEligible is IsAdult and HasRequirements and IsFinanciallyStable

If IsEligible:
    ApproveRental()
```

### 5. Avoid Double Negatives

```sfex
# Confusing
If not (not IsActive):
    DoSomething()

# Better
If IsActive:
    DoSomething()
```

## Common Patterns

### Range Check

```sfex
Story:
    Age is 25
    MinAge is 18
    MaxAge is 65

    InRange is Age >= MinAge and Age <= MaxAge
    Print InRange  # True
```

### Either-Or Check

```sfex
Story:
    Role is "admin"

    HasAccess is Role = "admin" or Role = "moderator"
    Print HasAccess  # True
```

### Safe Division

```sfex
Story:
    A is 10
    B is 0

    If B <> 0:
        Result is A / B
        Print Result
    Else:
        Print "Cannot divide by zero"
```

### Default Value Pattern

```sfex
Story:
    UserInput is ""

    # Use default if empty
    Name is If UserInput <> "" then UserInput else "Guest"
    Print Name  # "Guest"
```

### Null-Coalescing Pattern (with Option)

```sfex
Story:
    OptionalValue is None

    # Use UnwrapOr for default
    Value is OptionalValue.UnwrapOr(42)
    Print Value  # 42
```

## Examples

### Calculator

```sfex
Concept: Calculator
    To Calculate with Operator and A and B:
        When Operator:
            is "+":
                Return A + B
            is "-":
                Return A - B
            is "*":
                Return A * B
            is "/":
                If B = 0:
                    Print "Error: Division by zero"
                    Return 0
                Else:
                    Return A / B
            Otherwise:
                Print "Unknown operator"
                Return 0

Story:
    Create Calculator Called Calc

    Result1 is Calc.Calculate with "+" and 10 and 5
    Print "10 + 5 = " + Result1  # 15

    Result2 is Calc.Calculate with "*" and 10 and 5
    Print "10 * 5 = " + Result2  # 50
```

### Grade Calculator

```sfex
Story:
    Score is 85

    # Determine grade
    Grade is ""
    If Score >= 90:
        Grade is "A"
    Else If Score >= 80:
        Grade is "B"
    Else If Score >= 70:
        Grade is "C"
    Else If Score >= 60:
        Grade is "D"
    Else:
        Grade is "F"

    Print "Score: " + Score
    Print "Grade: " + Grade
```

### Validation

```sfex
Concept: Validator
    To ValidateAge with Age:
        IsValid is Age >= 0 and Age <= 150
        Return IsValid

    To ValidateEmail with Email:
        HasAt is Email contains "@"
        HasDot is Email contains "."
        NotEmpty is Email <> ""
        IsValid is NotEmpty and HasAt and HasDot
        Return IsValid

Story:
    Create Validator Called V

    If V.ValidateAge with 25:
        Print "Valid age"

    If V.ValidateEmail with "alice@example.com":
        Print "Valid email"
```

### Price Calculator

```sfex
Story:
    BasePrice is 100
    Quantity is 3
    TaxRate is 0.08
    DiscountRate is 0.10

    # Calculate subtotal
    Subtotal is BasePrice * Quantity

    # Apply discount
    Discount is Subtotal * DiscountRate
    AfterDiscount is Subtotal - Discount

    # Calculate tax
    Tax is AfterDiscount * TaxRate

    # Final total
    Total is AfterDiscount + Tax

    Print "Subtotal: $" + Subtotal      # $300
    Print "Discount: $" + Discount      # $30
    Print "After Discount: $" + AfterDiscount  # $270
    Print "Tax: $" + Tax                # $21.60
    Print "Total: $" + Total            # $291.60
```

## Summary

- **Arithmetic:** `+`, `-`, `*`, `/`, `%`
- **Comparison:** `=`, `<>`, `<`, `>`, `<=`, `>=`
- **Logical:** `and`, `or`, `not`
- **String:** `+` (concatenation), `contains`
- **List:** `contains`
- **Precedence:** Use parentheses for clarity
- **Mathematical honesty:** Exact decimal arithmetic by default

---

Next: [Comments](./comments.md) - Documenting your code

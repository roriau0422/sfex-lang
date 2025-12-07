# Mathematical Honesty

## The Floating Point Lie

One of the most shocking things new programmers discover:

```python
# Python (same in JavaScript, Java, C++, C#, Ruby, etc.)
result = 0.1 + 0.2
print(result)  # 0.30000000000000004 ❌
```

Teachers say "don't worry about it" or "you'll get used to it." But why should we?

## Why Does This Happen?

Most languages use IEEE 754 floating-point arithmetic, which represents numbers in binary:

- 0.1 in binary is `0.0001100110011001100...` (infinite repeating)
- 0.2 in binary is `0.001100110011001100...` (infinite repeating)
- Binary can't represent these exactly, so they're rounded
- Adding rounded numbers gives wrong answers

This was a reasonable choice in 1970 when:
- Memory was expensive
- CPUs had hardware floating-point units
- Alternative precision libraries were slow

But it's 2025 now. We can do better.

## The SFX Solution

SFX uses **arbitrary precision arithmetic** by default:

```sfex
Story:
    Result is 0.1 + 0.2
    Print Result  # 0.3 ✓

    # Financial calculations
    Price is 99.99
    Tax is Price * 0.08
    Total is Price + Tax
    Print Total  # 107.9892 (exact!)

    # Very large numbers
    Big is 123456789123456789123456789
    Bigger is Big * 2
    Print Bigger  # 246913578246913578246913578
```

## How It Works

SFX uses the `BigDecimal` type for the default `Number` type:

- Numbers are stored as integers with a scale factor
- 0.1 is stored as `1` with scale `-1` (meaning 1 × 10^-1)
- Operations maintain exact precision
- No rounding errors accumulate

### Example

```sfex
Story:
    # Exact decimal representation
    A is 0.1        # Stored exactly
    B is 0.2        # Stored exactly
    C is A + B      # 0.3 exactly

    # This always works:
    If C = 0.3:
        Print "Math works!"  # Always prints ✓
```

## When You Need Speed: FastNumber

For performance-critical code (games, physics, graphics), use `FastNumber`:

```sfex
Story:
    # IEEE 754 float for speed
    Speed is FastNumber(299792458.0)
    Time is FastNumber(2.5)
    Distance is Speed * Time

    # About 10x faster than Number
    # But you get float errors again:
    Test is FastNumber(0.1) + FastNumber(0.2)
    # 0.30000000000000004
```

**When to use FastNumber:**
- Physics simulations
- Graphics/game engines
- High-frequency calculations
- When performance > precision

**When to use Number (default):**
- Financial calculations
- User-facing math
- Counting/indexing
- When correctness matters
- Learning/teaching programming

## Real-World Impact

### Financial Software

```sfex
Concept: Invoice
    Subtotal, TaxRate, Tax, Total

    When Subtotal changes:
        Set This.Tax to This.Subtotal * This.TaxRate
        Set This.Total to This.Subtotal + This.Tax

Story:
    Create Invoice Called Bill
    Set Bill.Subtotal to 99.99
    Set Bill.TaxRate to 0.08

    # Exact: $107.9892
    # Float: $107.98919999999999 (wrong!)
    Print Bill.Total
```

With floats, multiply this error by millions of transactions, and you're off by thousands of dollars.

### Scientific Computing

```sfex
Story:
    # Sum a series: 1 + 1/2 + 1/3 + 1/4 + ...
    Sum is 0
    Repeat 10000 times with I:
        Sum is Sum + (1 / I)

    # SFX: Exact answer
    # Float: Accumulated rounding errors
    Print Sum
```

### Teaching Math

When teaching programming to kids, having `0.1 + 0.2 = 0.3` work correctly means:
- Less confusion
- Math matches what they learned in school
- One less "computers are weird" moment
- Focus on logic, not float quirks

## Performance Comparison

| Operation | Number (exact) | FastNumber (float) |
|-----------|---------------|-------------------|
| Addition | 100% | ~1000% (10x faster) |
| Multiplication | 100% | ~1000% |
| Division | 100% | ~1000% |

**With JIT compilation**, Number operations get optimized, narrowing the gap.

## Examples

### Banking

```sfex
Concept: BankAccount
    Balance

    To Deposit with Amount:
        Set This.Balance to This.Balance + Amount

    To Withdraw with Amount:
        Set This.Balance to This.Balance - Amount

Story:
    Create BankAccount Called Checking
    Set Checking.Balance to 0

    # Deposit cents
    Checking.Deposit with 0.01
    Repeat 100 times:
        Checking.Deposit with 0.01

    # Balance is exactly $1.01
    Print Checking.Balance  # 1.01 ✓
```

With floats, you'd get `1.0100000000000007` or similar.

### E-commerce

```sfex
Concept: ShoppingCart
    Items, Subtotal, Discount, Tax, Total

    To CalculateTotal:
        ItemsSum is 0
        For each Item in This.Items:
            ItemsSum is ItemsSum + Item["price"]

        Set This.Subtotal to ItemsSum
        DiscountAmount is This.Subtotal * This.Discount
        TaxableAmount is This.Subtotal - DiscountAmount
        TaxAmount is TaxableAmount * This.Tax
        Set This.Total to TaxableAmount + TaxAmount

Story:
    Create ShoppingCart Called Cart
    Set Cart.Items to [
        { name: "Item 1", price: 19.99 },
        { name: "Item 2", price: 29.99 },
        { name: "Item 3", price: 9.99 }
    ]
    Set Cart.Discount to 0.10  # 10% off
    Set Cart.Tax to 0.08       # 8% tax

    Cart.CalculateTotal
    Print "Total: $" + Cart.Total  # Exact to the cent!
```

## Best Practices

### 1. Use Number by Default

```sfex
# Good - exact
Price is 19.99
Tax is Price * 0.08
```

### 2. Use FastNumber for Performance

```sfex
# Physics engine
Concept: Particle
    X, Y, VX, VY

    To Update with DT:
        # Convert to FastNumber for speed
        FDT is FastNumber(DT)
        Set This.X to This.X + (This.VX * FDT)
        Set This.Y to This.Y + (This.VY * FDT)
```

### 3. Don't Mix Types Unnecessarily

```sfex
# Avoid mixing Number and FastNumber
A is 10
B is FastNumber(20.0)
C is A + B  # Works, but converts to FastNumber (loses precision)
```

### 4. Round for Display

```sfex
# For user display, round to appropriate precision
Price is 19.99
Tax is Price * 0.085  # 1.69915
Total is Price + Tax  # 21.68915

# Display as currency (2 decimal places)
DisplayTotal is Math.Round(Total * 100) / 100
Print "$" + DisplayTotal  # $21.69
```

## Summary

SFX's mathematical honesty means:

✓ **Correct by default** - Math works like school taught you
✓ **No surprises** - 0.1 + 0.2 = 0.3
✓ **Financial accuracy** - Pennies don't go missing
✓ **Easier to learn** - No float quirks to explain
✓ **FastNumber available** - Performance when you need it

Programming languages should make correctness easy and performance opt-in, not the other way around.

---

Next: [1-Based Indexing](./one-based-indexing.md) - Why lists start at 1

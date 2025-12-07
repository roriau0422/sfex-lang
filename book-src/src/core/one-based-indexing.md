# 1-Based Indexing

## The Zero-Based Confusion

Quick question: What's the first item in this list?

```
Shopping List:
1. Milk
2. Eggs
3. Bread
```

Everyone says "Milk" - it's item #1.

Now, in most programming languages:

```python
# Python, JavaScript, Java, C++, etc.
shopping = ["Milk", "Eggs", "Bread"]
first = shopping[0]  # 0? But we just said it's #1!
```

Why this disconnect?

## Historical Reasons

Zero-based indexing comes from C, where arrays are implemented as pointers:

```c
// C language
int array[5];
// array[0] means: "address of array + (0 * sizeof(int))"
// array[3] means: "address of array + (3 * sizeof(int))"
```

This was a **hardware optimization** from the 1970s. The first element is offset by zero bytes, so accessing it is slightly faster.

But that's an implementation detail! Why should humans care about memory offsets?

## The SFX Solution

SFX uses **1-based indexing** because that's how humans count:

```sfex
Story:
    Shopping is ["Milk", "Eggs", "Bread"]

    First is Shopping[1]   # Milk ✓
    Second is Shopping[2]  # Eggs ✓
    Third is Shopping[3]   # Bread ✓

    # The length matches the last index!
    Count is Shopping.Length  # 3
    Last is Shopping[Count]   # Bread ✓
```

## Why 1-Based is Better

### 1. Matches Human Counting

```sfex
Story:
    # Task: Print the first 3 items
    Items is ["A", "B", "C", "D", "E"]

    Repeat 3 times with I:
        Print Items[I]  # I goes 1, 2, 3 - perfect!
```

Compare to zero-based:
```python
# Confusing: Print items 0, 1, 2 to get the "first 3"
for i in range(3):  # range(3) means 0,1,2 - huh?
    print(items[i])
```

### 2. Length Matches Last Index

```sfex
Story:
    Numbers is [10, 20, 30, 40, 50]
    Count is Numbers.Length  # 5
    Last is Numbers[Count]   # 50 ✓ Makes sense!
```

Zero-based:
```python
numbers = [10, 20, 30, 40, 50]
count = len(numbers)  # 5
last = numbers[count]  # ERROR! Out of bounds!
last = numbers[count - 1]  # 50 (why -1?)
```

### 3. Fewer Off-By-One Errors

The infamous "off-by-one error" is often caused by zero-based thinking:

```python
# Bug-prone (0-based)
for i in range(len(items)):  # 0 to length-1
    # Easy to forget the -1 in range(start, end-1)

# Or:
for i in range(1, len(items)):  # Accidentally skips first item!
```

SFX:
```sfex
# Clear (1-based)
Repeat Items.Length times with I:
    Print Items[I]  # I goes 1, 2, 3... naturally
```

### 4. Clearer Slicing

```sfex
Story:
    Numbers is [10, 20, 30, 40, 50]

    # Get items 2 through 4
    Slice is Numbers.Slice(2, 4)  # [20, 30, 40] ✓
    # Start at 2, end at 4, includes both!
```

Compare to zero-based (Python):
```python
numbers = [10, 20, 30, 40, 50]
slice = numbers[1:4]  # [20, 30, 40]
# Start at 1 (second item), end BEFORE 4 - confusing!
```

## Common Scenarios

### Looping with Index

```sfex
Story:
    Names is ["Alice", "Bob", "Charlie"]

    # Natural counting
    Repeat Names.Length times with I:
        Print "Person " + I + ": " + Names[I]
    # Output:
    # Person 1: Alice
    # Person 2: Bob
    # Person 3: Charlie
```

### Finding an Index

```sfex
Story:
    Items is ["apple", "banana", "cherry"]
    Target is "banana"

    Found is 0
    Repeat Items.Length times with I:
        If Items[I] = Target:
            Found is I
            Break

    If Found > 0:
        Print "Found at position " + Found  # Position 2 ✓
```

### Accessing First and Last

```sfex
Story:
    Numbers is [5, 10, 15, 20, 25]

    First is Numbers[1]             # 5
    Last is Numbers[Numbers.Length] # 25

    # Middle element (for odd-length lists)
    Middle is Numbers[(Numbers.Length + 1) / 2]
```

## Frequently Asked Questions

### "But zero-based is more efficient!"

**No.** Modern compilers optimize both equally. The difference is nanoseconds.

SFX's JIT compiler generates the same machine code whether you write `array[1]` or `array[0]` - it just adjusts the offset.

### "Zero-based is standard everywhere!"

So were punch cards, GOTO statements, and null pointers. Just because something is common doesn't make it right.

Many successful languages use 1-based indexing:
- MATLAB (engineering/scientific computing)
- R (statistics)
- Lua (game scripting)
- Fortran (still used in supercomputing)

### "What about C interop?"

SFX's stdlib handles the conversion. When you call a C library, SFX adjusts indices automatically.

```sfex
# You write (1-based)
Result is SomeArray[1]

# SFX internally (0-based for C interop)
# result = some_array[0]
```

### "I'll get confused switching between languages!"

People switch between:
- Languages that start weeks on Sunday vs Monday
- Countries that drive on left vs right
- Date formats (MM/DD/YY vs DD/MM/YY)

Programmers are smart. You can handle lists starting at 1 in SFX and 0 in Python.

## Examples

### Ranking

```sfex
Concept: Leaderboard
    Scores

    To ShowRankings:
        # Sort in descending order
        Sorted is This.Scores.Sort("desc")

        Print "=== Rankings ==="
        Repeat Sorted.Length times with Position:
            Player is Sorted[Position]
            Print "#" + Position + ": " + Player["name"] + " - " + Player["score"]

Story:
    Create Leaderboard Called Board
    Set Board.Scores to [
        { name: "Alice", score: 95 },
        { name: "Bob", score: 87 },
        { name: "Charlie", score: 92 }
    ]

    Board.ShowRankings
    # Output:
    # === Rankings ===
    # #1: Alice - 95
    # #2: Charlie - 92
    # #3: Bob - 87
```

Notice how the rank matches the index naturally!

### Pagination

```sfex
Story:
    AllItems is [/* 100 items */]
    PageSize is 10
    CurrentPage is 1

    # Page 1: items 1-10
    # Page 2: items 11-20
    # etc.
    StartIndex is ((CurrentPage - 1) * PageSize) + 1
    EndIndex is CurrentPage * PageSize

    # Get items for current page
    PageItems is AllItems.Slice(StartIndex, EndIndex)
```

### Grid/Matrix

```sfex
Story:
    # 3x3 grid
    Grid is [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ]

    # Access row 2, column 3
    Value is Grid[2][3]  # 6 (second row, third item)

    # Human-readable coordinates
    Print "Row 1, Col 1: " + Grid[1][1]  # 1 ✓
```

## Migration from 0-Based Languages

If you're coming from Python/JavaScript/Java:

```python
# Python (0-based)
first = items[0]
second = items[1]
last = items[len(items) - 1]
```

```sfex
# SFX (1-based)
First is Items[1]
Second is Items[2]
Last is Items[Items.Length]
```

**Tip:** Think "first, second, third" not "zeroth, first, second."

## Best Practices

### Loop from 1 to Length

```sfex
# Good
Repeat List.Length times with I:
    Print List[I]
```

### Use For Each When You Don't Need Index

```sfex
# Even better when index isn't needed
For each Item in List:
    Print Item
```

### Check Bounds

```sfex
# Safe access
Index is 5
If Index >= 1 and Index <= List.Length:
    Value is List[Index]
Else:
    Print "Index out of range"
```

## Summary

SFX uses 1-based indexing because:

✓ **Natural** - Matches how humans count
✓ **Clearer** - First item is index 1, not 0
✓ **Consistent** - Length equals last index
✓ **Fewer errors** - Less off-by-one confusion
✓ **No performance cost** - JIT optimizes equally

Programming is for humans, not computers. Let's count like humans.

---

Next: [No Null Pointers](./no-null.md) - Safe defaults and the Option type

# Quick Start

Get up and running with SFX in 5 minutes!

## Hello World

Create a file called `hello.sfex`:

```sfex
Story:
    Print "Hello, SFX!"
```

Run it:

```bash
sfex run hello.sfex
# Output: Hello, SFX!
```

## Mathematical Honesty

SFX uses arbitrary precision arithmetic - no more float errors:

```sfex
Story:
    Result is 0.1 + 0.2
    Print Result  # 0.3 ✓ (not 0.30000000000000004)
```

## 1-Based Indexing

Lists start at 1, not 0:

```sfex
Story:
    Numbers is [10, 20, 30]
    First is Numbers[1]   # 10 (first element)
    Second is Numbers[2]  # 20
    Third is Numbers[3]   # 30
    Print "First: " + First
```

## Variables and Assignment

Use `is` for assignment:

```sfex
Story:
    Name is "Alice"
    Age is 25
    Score is 95.5
    IsActive is True

    Print Name
    Print "Age: " + Age
```

## Control Flow

### If/Else

```sfex
Story:
    Score is 85

    If Score >= 90:
        Print "A grade"
    Else If Score >= 80:
        Print "B grade"
    Else:
        Print "Keep trying"
```

### Loops

```sfex
Story:
    # Repeat a fixed number of times
    Repeat 3 times:
        Print "Hello!"

    # Loop over a list
    Colors is ["red", "green", "blue"]
    For each Color in Colors:
        Print Color
```

## Working with Lists

```sfex
Story:
    # Create a list
    Items is [1, 2, 3, 4, 5]

    # Access elements (1-based)
    First is Items[1]

    # Get length
    Count is Items.Length
    Print "Count: " + Count  # 5

    # Add elements
    Items.Add(6)
```

## Working with Maps

```sfex
Story:
    # Create a map
    User is { name: "Bob", age: 30, city: "New York" }

    # Access values
    Name is User["name"]
    Age is User["age"]

    Print "Name: " + Name
    Print "Age: " + Age
```

## Creating Objects (Concepts)

```sfex
Concept: Person
    Name
    Age

    To Greet:
        Print "Hello, I'm " + This.Name

    To Birthday:
        Set This.Age to This.Age + 1

Story:
    Create Person Called Alice
    Set Alice.Name to "Alice"
    Set Alice.Age to 30

    Alice.Greet           # Hello, I'm Alice
    Alice.Birthday
    Print Alice.Age       # 31
```

## File Operations

```sfex
Story:
    # Write to a file
    Content is "Hello from SFX!"
    File.Write("output.txt", Content)

    # Read from a file
    Data is File.Read("output.txt")
    Print Data
```

## HTTP Requests

```sfex
Story:
    # Make an HTTP GET request
    Response is HTTP.Get("https://api.github.com/users/octocat")

    Print Response["Status"]  # 200
    Print Response["Body"]    # JSON response
```

## What's Next?

Now that you know the basics, dive deeper:

- [Your First Program](./first-program.md) - Build a complete application
- [Core Concepts](../core/why-sfx.md) - Understand SFX's philosophy
- [Language Syntax](../syntax/basics.md) - Complete syntax reference
- [Examples](../examples/hello-world.md) - More code examples

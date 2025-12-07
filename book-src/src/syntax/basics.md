# Basic Syntax

SFX uses a clean, readable syntax inspired by natural language. This chapter covers the fundamental syntax rules.

## Program Structure

Every SFX program has a `Story` block - the main entry point:

```sfex
Story:
    Print "Hello, SFX!"
```

A complete program can include:
- **Concepts** - Classes/objects (defined before Story)
- **Situations** - Context modifiers (defined before Story)
- **Story** - Main entry point (required)

```sfex
# Complete program structure
Concept: MyClass
    Field1, Field2

Situation: MyContext
    Adjust MyClass:
        # Adjustments here

Story:
    # Main program code here
    Print "Starting program..."
```

## Indentation

SFX uses **significant indentation** like Python:

- Use **4 spaces** per indentation level
- **DO NOT use tabs** (use spaces only)
- Consistent indentation defines code blocks

```sfex
Story:
    If True:
        Print "This is indented"
        Print "So is this"
    Print "Back to outer level"
```

## Comments

Use `#` for single-line comments:

```sfex
# This is a comment
Story:
    Print "Hello"  # Inline comment
    # Another comment
```

**Note:** SFX currently supports only single-line comments. Multi-line comments are not yet supported.

## Statements

Statements in SFX typically end at the end of the line:

```sfex
Story:
    Name is "Alice"
    Age is 25
    Print Name
```

## Keywords

SFX uses descriptive English keywords:

| Category | Keywords |
|----------|----------|
| Structure | `Story`, `Concept`, `Situation` |
| Assignment | `is`, `Set`, `to` |
| Control Flow | `If`, `Else`, `Repeat`, `For`, `each`, `in`, `When`, `Otherwise` |
| Object Creation | `Create`, `Called` |
| Methods | `To`, `with`, `and`, `Return` |
| Context | `Switch`, `on`, `off`, `Adjust`, `Proceed` |
| Error Handling | `Try`, `Catch`, `Always` |
| Concurrency | `Do`, `in`, `background` |
| Logic | `True`, `False`, `and`, `or`, `not` |
| Loops | `Break`, `Continue`, `times`, `while` |

## Case Sensitivity

SFX keywords are **case-sensitive**:

```sfex
Story:
    # Correct
    Print "Hello"

    # Wrong (will cause error)
    print "Hello"  # lowercase 'p'
    PRINT "Hello"  # uppercase
```

Variable and concept names are also case-sensitive:

```sfex
Story:
    Name is "Alice"
    name is "Bob"
    # 'Name' and 'name' are different variables
```

## Line Continuation

Long statements can be continued on the next line within certain contexts:

```sfex
# Lists and maps can span multiple lines
Items is [
    "Item 1",
    "Item 2",
    "Item 3"
]

User is {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}
```

## Naming Conventions

### Variables and Fields

Use **PascalCase** for variable and field names:

```sfex
Story:
    FirstName is "Alice"
    LastName is "Smith"
    UserAge is 25
    IsActive is True
```

### Concepts

Use **PascalCase** for concept names:

```sfex
Concept: Person
Concept: ShoppingCart
Concept: HttpClient
```

### Methods

Use **PascalCase** for method names:

```sfex
Concept: Calculator
    To Add with X and Y:
        Return X + Y

    To CalculateTotal:
        # Method code
```

### Situations

Use **PascalCase** for situation names:

```sfex
Situation: AdminMode
Situation: DebugMode
Situation: VIPCustomer
```

## Best Practices

### 1. Use Descriptive Names

```sfex
# Good
UserName is "Alice"
ShoppingCart is []
TotalPrice is 99.99

# Less clear
UN is "Alice"
SC is []
TP is 99.99
```

### 2. Keep Indentation Consistent

```sfex
# Good
Story:
    If Condition:
        DoSomething()
        DoSomethingElse()

# Bad (inconsistent indentation)
Story:
    If Condition:
      DoSomething()
        DoSomethingElse()  # Error!
```

### 3. One Statement Per Line

```sfex
# Good
Name is "Alice"
Age is 25

# Possible but discouraged
Name is "Alice", Age is 25  # Works only with Set statements
```

### 4. Group Related Code

```sfex
Story:
    # Configuration
    Host is "localhost"
    Port is 8080

    # User setup
    Create User Called Admin
    Set Admin.Name to "Administrator"
    Set Admin.Role to "admin"

    # Start server
    Server.Start with Host and Port
```

### 5. Use Comments for Clarity

```sfex
Story:
    # Calculate tax (10% rate)
    Price is 100
    Tax is Price * 0.1
    Total is Price + Tax

    # Display result
    Print "Total with tax: $" + Total
```

## Example: Complete Program

```sfex
# User Management System

Concept: User
    Name, Email, Role, IsActive

    To Activate:
        Set This.IsActive to True
        Print This.Name + " has been activated"

    To Deactivate:
        Set This.IsActive to False
        Print This.Name + " has been deactivated"

    To UpdateEmail with NewEmail:
        Set This.Email to NewEmail
        Print "Email updated to: " + NewEmail

Situation: AdminMode
    Adjust User:
        To UpdateEmail with NewEmail:
            # Admin can force update
            Set This.Email to NewEmail
            Print "[ADMIN] Email forcefully updated to: " + NewEmail

Story:
    # Create users
    Create User Called Alice
    Set Alice.Name to "Alice Smith"
    Set Alice.Email to "alice@example.com"
    Set Alice.Role to "user"
    Set Alice.IsActive to True

    Create User Called Bob
    Set Bob.Name to "Bob Jones"
    Set Bob.Email to "bob@example.com"
    Set Bob.Role to "admin"
    Set Bob.IsActive to False

    # Activate Bob
    Bob.Activate

    # Update Alice's email
    Alice.UpdateEmail with "alice.smith@example.com"

    # Admin mode - force update
    Switch on AdminMode
    Bob.UpdateEmail with "admin@example.com"
    Switch off AdminMode

    Print "=== User Summary ==="
    Print Alice.Name + " - " + Alice.Email + " (Active: " + Alice.IsActive + ")"
    Print Bob.Name + " - " + Bob.Email + " (Active: " + Bob.IsActive + ")"
```

## What's Next?

Now that you understand the basics, learn about:

- [Variables and Assignment](./variables.md) - How to work with variables
- [Data Types](./data-types.md) - Numbers, strings, lists, maps, and more
- [Operators](./operators.md) - Arithmetic, comparison, logical operators
- [Comments](./comments.md) - Documenting your code
- [Control Flow](../control-flow/if-else.md) - Making decisions and loops

# Your First Program

Let's build a complete To-Do list application to learn SFX fundamentals.

## The To-Do List App

Create a file called `todo.sfex`:

```sfex
# To-Do List Application

Concept: TodoItem
    Title
    IsDone

    To Display:
        If This.IsDone:
            Print "✓ " + This.Title
        Else:
            Print "☐ " + This.Title

Concept: TodoList
    Items

    To Initialize:
        Set This.Items to []

    To AddTask with Title:
        Create TodoItem Called NewItem
        Set NewItem.Title to Title
        Set NewItem.IsDone to False
        Set This.Items to This.Items + [NewItem]
        Print "Added: " + Title

    To ShowAll:
        Print "\n=== My To-Do List ==="
        If This.Items.Length = 0:
            Print "No tasks yet!"
        Else:
            For each Item in This.Items:
                Item.Display

    To CompleteTask with Index:
        If Index >= 1 and Index <= This.Items.Length:
            Task is This.Items[Index]
            Set Task.IsDone to True
            Print "✓ Completed: " + Task.Title
            # Update the item in the list
            NewItems is []
            Counter is 1
            For each Item in This.Items:
                If Counter = Index:
                    Set NewItems to NewItems + [Task]
                Else:
                    Set NewItems to NewItems + [Item]
                Set Counter to Counter + 1
            Set This.Items to NewItems
        Else:
            Print "Invalid task number!"

Story:
    # Create the to-do list
    Create TodoList Called MyTodos
    MyTodos.Initialize

    # Add some tasks
    MyTodos.AddTask with "Buy groceries"
    MyTodos.AddTask with "Write documentation"
    MyTodos.AddTask with "Learn SFX"
    MyTodos.AddTask with "Build an app"

    # Show all tasks
    MyTodos.ShowAll

    # Complete some tasks
    Print "\n--- Completing tasks ---"
    MyTodos.CompleteTask with 1
    MyTodos.CompleteTask with 3

    # Show updated list
    MyTodos.ShowAll
```

## Run the Program

```bash
sfex run todo.sfex
```

Expected output:

```
Added: Buy groceries
Added: Write documentation
Added: Learn SFX
Added: Build an app

=== My To-Do List ===
☐ Buy groceries
☐ Write documentation
☐ Learn SFX
☐ Build an app

--- Completing tasks ---
✓ Completed: Buy groceries
✓ Completed: Learn SFX

=== My To-Do List ===
✓ Buy groceries
☐ Write documentation
✓ Learn SFX
☐ Build an app
```

## What Did We Learn?

### 1. Concepts (Classes)

```sfex
Concept: TodoItem
    Title          # Field declaration
    IsDone

    To MarkDone:   # Method definition
        # Method body
```

Concepts are SFX's version of classes. They contain:
- **Fields** - Data storage (Title, IsDone)
- **Methods** - Functions that operate on the object

### 2. Creating Instances

```sfex
Create TodoList Called MyTodos
```

The `Create` statement creates a new instance of a Concept.

### 3. Setting Field Values

```sfex
Set NewItem.Title to Title
Set NewItem.IsDone to False
```

The `Set` statement modifies object fields.

### 4. Method Calls

```sfex
MyTodos.AddTask with "Buy groceries"
Item.Display
```

- Methods without parameters: `Item.Display`
- Methods with parameters: `MyTodos.AddTask with "Buy groceries"`

### 5. This Keyword

```sfex
To AddTask with Title:
    Set This.Items to This.Items + [NewItem]  # 'This' refers to current object
```

Inside methods, `This` refers to the current instance.

### 6. Lists

```sfex
Set This.Items to []                      # Create empty list
Set This.Items to This.Items + [NewItem]  # Add element (concatenation)
Count is This.Items.Length                # Get length
Task is This.Items[Index]                 # Access by index (1-based!)
```

Lists in SFX:
- Created with `[]`
- **1-based indexing** - first element is at index 1
- Dynamic - grow by concatenating with `+`
- Have `.Length` property

### 7. Control Flow

```sfex
If This.Items.Length = 0:
    Print "No tasks yet!"
Else:
    For each Item in This.Items:
        Item.Display
```

- `If/Else` for conditionals
- `For each` for iterating over lists

### 8. Parameters

```sfex
To AddTask with Title:
    # 'Title' is the parameter
```

Methods can take parameters using `with ParameterName`.

### 9. Value Semantics

```sfex
To CompleteTask with Index:
    Task is This.Items[Index]   # Gets a copy of the item
    Set Task.IsDone to True     # Modifies the copy
    # Need to update the original in the list
    NewItems is []
    Counter is 1
    For each Item in This.Items:
        If Counter = Index:
            Set NewItems to NewItems + [Task]  # Use modified copy
        Else:
            Set NewItems to NewItems + [Item]
        Set Counter to Counter + 1
    Set This.Items to NewItems
```

**Important**: When you get an item from a list, you get a copy, not a reference. To update an item in a list:
1. Get the item (creates a copy)
2. Modify the copy
3. Rebuild the list with the modified copy

This "value semantics" approach ensures data safety - changes to copies don't accidentally affect originals elsewhere.

## Enhancing the Program

### Add Task Priority

```sfex
Concept: TodoItem
    Title
    IsDone
    Priority  # New field: 1=High, 2=Medium, 3=Low

    To Display:
        PriorityText is ""
        When This.Priority:
            is 1:
                PriorityText is "[HIGH] "
            is 2:
                PriorityText is "[MEDIUM] "
            is 3:
                PriorityText is "[LOW] "

        If This.IsDone:
            Print "✓ " + PriorityText + This.Title
        Else:
            Print "☐ " + PriorityText + This.Title
```

### Add Due Dates

```sfex
Concept: TodoItem
    Title
    IsDone
    DueDate

    To IsOverdue:
        Now is Time.Now()
        If This.DueDate < Now:
            Return True
        Else:
            Return False
```

## What's Next?

You've learned the fundamentals! Continue with:

- [Core Concepts](../core/why-sfx.md) - Deep dive into SFX philosophy
- [Language Syntax](../syntax/basics.md) - Complete syntax reference
- [Object-Oriented Programming](../oop/concepts.md) - Advanced OOP features
- [Reactive Programming](../reactive/when-observers.md) - Self-healing data
- [Context-Oriented Programming](../cop/situations.md) - Dynamic behavior changes
- [Examples](../examples/hello-world.md) - More practical examples

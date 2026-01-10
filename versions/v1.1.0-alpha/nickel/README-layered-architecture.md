# Layered Nickel Architecture: Primitives → Libraries → Typed Collections

This architecture builds complex directory contracts from discrete merged schema files, providing typed collections to directories while abstracting away primitive implementations.

## 🏗️ Architecture Overview

```
Primitives (Discrete Building Blocks)
├── contract_primitives.ncl    - Basic contract interfaces
├── hydration_primitives.ncl   - Runtime configuration blocks
├── edge_primitives.ncl        - Relationship semantics
└── layer_primitives.ncl       - Architectural layer definitions

Libraries (Merged Compositions)
├── contract_library.ncl       - Complete contract system
├── hydration_library.ncl      - Full hydration configurations
├── edge_library.ncl           - Relationship validation & types
└── layer_library.ncl          - Layer validation & functions

Composers (Typed Collections)
└── directory_composer.ncl     - Abstracted interfaces for directories

Directories (Typed Consumers)
├── architecture/_directory.ncl  - Uses ArchitectureInterface
├── infrastructure/_directory.ncl - Uses InfrastructureInterface
└── codegen/_directory.ncl       - Uses CodegenInterface
```

## 🎯 Design Principles

### **1. Primitive Isolation**
- Each primitive is a focused, discrete building block
- No dependencies between primitives
- Easy to test and modify individually

### **2. Library Composition**
- Libraries merge primitives into cohesive systems
- Add library-specific enhancements and validation
- Provide unified APIs for related functionality

### **3. Typed Abstraction**
- Composers create typed collections from merged libraries
- Directories get strongly-typed interfaces
- Implementation details hidden behind abstractions

### **4. Directory Focus**
- Directories contain only domain-specific logic
- Typed collections provide all necessary tools
- No knowledge of underlying primitives or merging

## 🔧 Usage Patterns

### **Directory Contract Creation**
```nickel
# Directory imports typed collection
let Composer = import "../composers/directory_composer.ncl" in

# Uses abstracted interface - no primitive knowledge needed
Composer.architecture.Contract.compose {
  extract_from_raw = my_extract_function,
  validate_pure = my_validate_function
}
```

### **Library Enhancement**
```nickel
# Libraries add value beyond merged primitives
ContractPrimitives.InterfacePrimitives &
ContractPrimitives.FileStructurePrimitives & {
  # Library-specific enhancements
  enhanced_test_runner = fun contract test_cases => /* ... */
}
```

### **Primitive Composition**
```nickel
# Primitives are pure building blocks
InterfacePrimitives = {
  extract_from_raw = fun schema => schema,
  validate_pure = fun schema => {valid = true, errors = [], warnings = []}
}
```

## 🎨 Nickel Pattern Alignment

### **✅ "Composable Data"**
- Primitives → Libraries → Collections all use record merging (`&`)
- Each layer builds upon the previous through composition

### **✅ "Modular Configurations"**
- Primitives are modular building blocks
- Libraries provide modular functionality groups
- Collections offer modular interfaces

### **✅ "Infrastructure as Code"**
- Generates complex validation infrastructure from primitives
- Supports build-time composition and validation
- Enables automated schema processing pipelines

## 📊 Benefits Achieved

| **Aspect** | **Before** | **After** | **Improvement** |
|------------|------------|-----------|-----------------|
| **Complexity Management** | Monolithic contracts | Layered composition | **Modular** |
| **Abstraction Level** | Direct primitive usage | Typed collections | **Abstracted** |
| **Maintainability** | Coupled components | Isolated layers | **Decoupled** |
| **Testability** | Hard to test | Each layer testable | **Improved** |
| **Reusability** | Limited | Primitives reusable | **Enhanced** |

## 🚀 Result

**Directories get powerful, typed collections without knowing about implementation details.** The architecture supports complex schema validation while maintaining clean separation between primitives, composition logic, and consumer interfaces.

This creates a **maintainable, extensible, and strongly-typed** configuration system built on Nickel's core strengths of composable data and modular configurations.
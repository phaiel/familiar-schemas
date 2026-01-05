#!/bin/bash

# Comprehensive audit of references to moved schemas
# This will find all broken references after reorganization

echo "🔍 COMPREHENSIVE REFERENCE AUDIT"
echo "=================================="

cd versions/latest/json-schema

# Track moved schemas and their new locations
declare -A moved_schemas=(
    # Architecture schemas (ecs/ → architecture/)
    ["Action.meta.schema.json"]="architecture/meta/Action.meta.schema.json"
    ["System.meta.schema.json"]="architecture/meta/System.meta.schema.json"
    ["Component.meta.schema.json"]="architecture/meta/Component.meta.schema.json"
    ["Technique.meta.schema.json"]="architecture/meta/Technique.meta.schema.json"
    ["Resource.meta.schema.json"]="architecture/meta/Resource.meta.schema.json"
    ["Node.meta.schema.json"]="architecture/meta/Node.meta.schema.json"
    ["Queue.meta.schema.json"]="architecture/meta/Queue.meta.schema.json"
    ["Agentic.meta.schema.json"]="architecture/meta/Agentic.meta.schema.json"
    ["Auth.meta.schema.json"]="architecture/meta/Auth.meta.schema.json"
    ["Conversation.meta.schema.json"]="architecture/meta/Conversation.meta.schema.json"
    ["EntitiesAPI.meta.schema.json"]="architecture/meta/EntitiesAPI.meta.schema.json"
    ["Entity.meta.schema.json"]="architecture/meta/Entity.meta.schema.json"
    ["UI.meta.schema.json"]="architecture/meta/UI.meta.schema.json"
    ["Database.meta.schema.json"]="architecture/meta/Database.meta.schema.json"
    ["Config.meta.schema.json"]="architecture/meta/Config.meta.schema.json"
    ["Contract.meta.schema.json"]="architecture/meta/Contract.meta.schema.json"
    ["Tool.meta.schema.json"]="architecture/meta/Tool.meta.schema.json"
    ["Persistence.meta.schema.json"]="architecture/meta/Persistence.meta.schema.json"
    ["Codegen.meta.schema.json"]="architecture/meta/RustCodegen.meta.schema.json"

    # Category schemas
    ["ComputeCategory.schema.json"]="architecture/categories/ComputeCategory.schema.json"
    ["ExecutionModel.schema.json"]="architecture/enums/ExecutionModel.schema.json"
    ["SideEffect.schema.json"]="architecture/enums/SideEffect.schema.json"
    ["Reliability.schema.json"]="architecture/enums/Reliability.schema.json"
    ["ToolingCategory.schema.json"]="architecture/categories/ToolingCategory.schema.json"
    ["Casing.schema.json"]="architecture/enums/Casing.schema.json"
    ["EnumRepr.schema.json"]="architecture/enums/EnumRepr.schema.json"

    # Tooling schemas
    ["DatabaseTooling.schema.json"]="architecture/tooling/DatabaseTooling.schema.json"
    ["LlmTooling.schema.json"]="architecture/tooling/LlmTooling.schema.json"
    ["SerializationTooling.schema.json"]="architecture/tooling/SerializationTooling.schema.json"
    ["ToolingReference.schema.json"]="architecture/references/ToolingReference.schema.json"
    ["ToolingResolution.schema.json"]="architecture/tooling/ToolingResolution.schema.json"
    ["LibraryReference.schema.json"]="architecture/references/LibraryReference.schema.json"

    # Step schemas
    ["CallStep.meta.schema.json"]="architecture/steps/CallStep.meta.schema.json"
    ["SwitchStep.meta.schema.json"]="architecture/steps/SwitchStep.meta.schema.json"
    ["MapStep.meta.schema.json"]="architecture/steps/MapStep.meta.schema.json"
    ["ParallelStep.meta.schema.json"]="architecture/steps/ParallelStep.meta.schema.json"
    ["TransformStep.meta.schema.json"]="architecture/steps/TransformStep.meta.schema.json"
    ["Step.meta.schema.json"]="architecture/steps/Step.meta.schema.json"

    # Reference schemas
    ["SchemaRef.meta.schema.json"]="architecture/references/SchemaRef.meta.schema.json"
)

BROKEN_TOTAL=0
FIXED_TOTAL=0

echo "Checking references to moved schemas..."
echo ""

for schema in "${!moved_schemas[@]}"; do
    new_path="${moved_schemas[$schema]}"

    # Find all references to this schema (any path)
    refs=$(grep -r "$schema" . --include="*.json" | wc -l)

    if [[ $refs -gt 0 ]]; then
        echo "📄 $schema ($refs references)"

        # Check if any references still point to old locations
        old_refs=$(grep -r "../ecs/$schema\|../$schema" . --include="*.json" | wc -l)

        if [[ $old_refs -gt 0 ]]; then
            echo "   ❌ $old_refs BROKEN refs to old paths"
            ((BROKEN_TOTAL += old_refs))
        else
            echo "   ✅ All refs updated"
        fi

        # Verify the new path exists
        if [[ ! -f "$new_path" ]]; then
            echo "   ⚠️  WARNING: New path $new_path does not exist!"
        fi

        echo ""
    fi
done

echo "=================================="
echo "SUMMARY:"
echo "- Total broken references found: $BROKEN_TOTAL"
if [[ $BROKEN_TOTAL -eq 0 ]]; then
    echo "✅ ALL REFERENCES APPEAR TO BE FIXED"
else
    echo "❌ BROKEN REFERENCES DETECTED - RUN fix_refs.sh"
fi

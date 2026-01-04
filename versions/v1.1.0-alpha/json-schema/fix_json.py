#!/usr/bin/env python3
import os
import json
import re

def fix_json_file(filepath):
    """Fix common JSON syntax errors in a file"""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        # Fix missing commas between properties
        # Pattern: "type": "string""title": "Foo" -> "type": "string","title": "Foo"
        content = re.sub(r'("(?:[^"\\]|\\.)*")\s*("(?:[^"\\]|\\.)*")', r'\1,\2', content)
        
        # Fix trailing commas before closing braces/brackets
        # Pattern: "value": 123, } -> "value": 123 }
        content = re.sub(r',\s*([}\]])', r'\1', content)
        
        # Write back
        with open(filepath, 'w') as f:
            f.write(content)
        
        # Test if it's valid now
        try:
            json.loads(content)
            print(f"✓ Fixed: {filepath}")
            return True
        except json.JSONDecodeError as e:
            print(f"✗ Still invalid ({e}): {filepath}")
            return False
            
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    count = 0
    for root, dirs, files in os.walk('.'):
        for file in files:
            if file.endswith('.json'):
                filepath = os.path.join(root, file)
                if fix_json_file(filepath):
                    count += 1
    
    print(f"\nFixed {count} JSON files")

if __name__ == '__main__':
    main()

#!/bin/bash

echo "Testing nullable timestamp fields migration..."

# Run the migration
echo "Running migration..."
cargo run --bin migrator up

if [ $? -eq 0 ]; then
    echo "✅ Migration completed successfully"
else
    echo "❌ Migration failed"
    exit 1
fi

echo ""
echo "Migration test completed!"

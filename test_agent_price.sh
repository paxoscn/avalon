#!/bin/bash

# Test script for agent price feature

echo "Testing Agent Price Feature"
echo "============================"
echo ""

# Check if the migration file exists
echo "1. Checking migration file..."
if [ -f "src/infrastructure/database/migrations/m20241127_000002_add_price_to_agents.rs" ]; then
    echo "✓ Migration file exists"
else
    echo "✗ Migration file not found"
    exit 1
fi

# Check if the field is added to domain entity
echo ""
echo "2. Checking domain entity..."
if grep -q "price: Option<Decimal>" "src/domain/entities/agent.rs"; then
    echo "✓ Field added to domain entity"
else
    echo "✗ Field not found in domain entity"
    exit 1
fi

# Check if the field is added to database entity
echo ""
echo "3. Checking database entity..."
if grep -q "price: Option<Decimal>" "src/infrastructure/database/entities/agent.rs"; then
    echo "✓ Field added to database entity"
else
    echo "✗ Field not found in database entity"
    exit 1
fi

# Check if the field is added to DTOs
echo ""
echo "4. Checking DTOs..."
if grep -q "price: Option<rust_decimal::Decimal>" "src/application/dto/agent_dto.rs"; then
    echo "✓ Field added to DTOs"
else
    echo "✗ Field not found in DTOs"
    exit 1
fi

# Check if update_price method exists
echo ""
echo "5. Checking update_price method..."
if grep -q "pub fn update_price" "src/domain/entities/agent.rs"; then
    echo "✓ update_price method exists"
else
    echo "✗ update_price method not found"
    exit 1
fi

# Check if rust_decimal dependency is added
echo ""
echo "6. Checking rust_decimal dependency..."
if grep -q "rust_decimal" "Cargo.toml"; then
    echo "✓ rust_decimal dependency added"
else
    echo "✗ rust_decimal dependency not found"
    exit 1
fi

echo ""
echo "============================"
echo "All checks passed! ✓"
echo ""
echo "Next steps:"
echo "1. Run 'cargo build' to compile the project"
echo "2. Run database migrations to add the new column"
echo "3. Test the API endpoints with price field"

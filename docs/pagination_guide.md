# Pagination Guide

## Overview

This guide provides comprehensive information about pagination in the Agent Platform API. All list endpoints follow a standardized pagination pattern to ensure consistency and ease of use.

## Quick Start

### Basic Pagination Request

```bash
GET /api/flows?page=1&limit=20
```

### Basic Pagination Response

```json
{
  "data": [...],
  "page": 1,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

## Pagination Parameters

### page (Query Parameter)

- **Type**: Integer
- **Required**: No
- **Default**: `1`
- **Minimum**: `1`
- **Description**: The page number to retrieve (1-based indexing)

**Examples:**
- `?page=1` - First page
- `?page=2` - Second page
- `?page=10` - Tenth page

**Important Notes:**
- Page numbers start from 1 (not 0) for user-friendly API design
- Requesting a page beyond available data returns an empty array (not an error)
- Page 0 is treated as page 1

### limit (Query Parameter)

- **Type**: Integer
- **Required**: No
- **Default**: `20`
- **Minimum**: `1`
- **Maximum**: `100`
- **Description**: The number of items to return per page

**Examples:**
- `?limit=10` - 10 items per page
- `?limit=50` - 50 items per page
- `?limit=100` - Maximum 100 items per page

**Important Notes:**
- Values less than 1 will return a validation error
- Values greater than 100 will return a validation error
- Some endpoints use `page_size` instead of `limit` (see Legacy Endpoints section)

## Response Format

All paginated endpoints return responses in this standardized format:

```json
{
  "data": [
    // Array of items for the current page
  ],
  "page": 1,           // Current page number (1-based)
  "limit": 20,         // Items per page
  "total": 150,        // Total number of items across all pages
  "total_pages": 8     // Total number of pages
}
```

### Response Fields

#### data
- **Type**: Array
- **Description**: The items for the current page
- **Notes**: Empty array if no items exist or page is beyond available data

#### page
- **Type**: Integer
- **Description**: The current page number (1-based)
- **Notes**: Matches the requested page parameter

#### limit
- **Type**: Integer
- **Description**: The number of items per page
- **Notes**: Matches the requested limit parameter

#### total
- **Type**: Integer
- **Description**: The total count of all items matching the query
- **Notes**: Useful for calculating progress and displaying "X of Y" information

#### total_pages
- **Type**: Integer
- **Description**: The total number of pages available
- **Formula**: `ceil(total / limit)`
- **Notes**: Use this to determine if more pages are available

## Common Use Cases

### 1. Fetching All Items

To retrieve all items across multiple pages:

```javascript
async function fetchAllFlows(token) {
  const allFlows = [];
  let page = 1;
  let hasMorePages = true;

  while (hasMorePages) {
    const response = await fetch(
      `http://localhost:8080/api/flows?page=${page}&limit=100`,
      {
        headers: { 'Authorization': `Bearer ${token}` }
      }
    );
    
    const data = await response.json();
    allFlows.push(...data.data);
    
    hasMorePages = page < data.total_pages;
    page++;
  }

  return allFlows;
}
```

### 2. Displaying Page Numbers

Calculate which page numbers to display in a UI:

```javascript
function calculatePageNumbers(currentPage, totalPages, maxVisible = 5) {
  const pages = [];
  const halfVisible = Math.floor(maxVisible / 2);
  
  let startPage = Math.max(1, currentPage - halfVisible);
  let endPage = Math.min(totalPages, startPage + maxVisible - 1);
  
  // Adjust start if we're near the end
  if (endPage - startPage < maxVisible - 1) {
    startPage = Math.max(1, endPage - maxVisible + 1);
  }
  
  for (let i = startPage; i <= endPage; i++) {
    pages.push(i);
  }
  
  return pages;
}

// Example: currentPage=5, totalPages=10, maxVisible=5
// Returns: [3, 4, 5, 6, 7]
```

### 3. Showing Progress

Display pagination progress to users:

```javascript
function getPaginationInfo(page, limit, total) {
  const startItem = (page - 1) * limit + 1;
  const endItem = Math.min(page * limit, total);
  
  return `Showing ${startItem}-${endItem} of ${total} items`;
}

// Example: page=2, limit=20, total=150
// Returns: "Showing 21-40 of 150 items"
```

### 4. Infinite Scroll

Implement infinite scroll pagination:

```javascript
class InfiniteScrollPagination {
  constructor(endpoint, token) {
    this.endpoint = endpoint;
    this.token = token;
    this.page = 1;
    this.limit = 20;
    this.hasMore = true;
    this.loading = false;
  }

  async loadMore() {
    if (this.loading || !this.hasMore) return [];

    this.loading = true;
    
    try {
      const response = await fetch(
        `${this.endpoint}?page=${this.page}&limit=${this.limit}`,
        {
          headers: { 'Authorization': `Bearer ${this.token}` }
        }
      );
      
      const data = await response.json();
      
      this.hasMore = this.page < data.total_pages;
      this.page++;
      
      return data.data;
    } finally {
      this.loading = false;
    }
  }
}
```

## Validation and Error Handling

### Validation Rules

The API enforces these validation rules:

1. **Page must be ≥ 1**
   - Invalid: `?page=0`
   - Valid: `?page=1`

2. **Limit must be between 1 and 100**
   - Invalid: `?limit=0`, `?limit=150`
   - Valid: `?limit=1`, `?limit=100`

### Error Responses

#### Invalid Limit (Too Small)

Request:
```bash
GET /api/flows?page=1&limit=0
```

Response (400 Bad Request):
```json
{
  "error": "Limit must be greater than 0",
  "timestamp": "2025-10-15T10:30:00Z"
}
```

#### Invalid Limit (Too Large)

Request:
```bash
GET /api/flows?page=1&limit=150
```

Response (400 Bad Request):
```json
{
  "error": "Limit cannot exceed 100",
  "timestamp": "2025-10-15T10:30:00Z"
}
```

#### Page Beyond Available Data

Request:
```bash
GET /api/flows?page=999&limit=20
```

Response (200 OK):
```json
{
  "data": [],
  "page": 999,
  "limit": 20,
  "total": 150,
  "total_pages": 8
}
```

**Note:** This is not an error - the API returns an empty array when the requested page is beyond available data.

## Performance Considerations

### Optimal Page Sizes

- **Small datasets (< 100 items)**: Use `limit=50` or `limit=100` to minimize requests
- **Medium datasets (100-1000 items)**: Use `limit=20` or `limit=50` for balanced performance
- **Large datasets (> 1000 items)**: Use `limit=20` (default) to avoid memory issues

### Deep Pagination

Be aware that requesting very high page numbers (e.g., page 1000) can be slower due to database offset operations. Consider these alternatives:

1. **Use filters** to narrow down results before paginating
2. **Implement cursor-based pagination** for very large datasets (future enhancement)
3. **Cache frequently accessed pages** on the client side

### Combining Filters with Pagination

Always apply filters before pagination to improve performance:

```bash
# Good: Filter first, then paginate
GET /api/audit/logs?user_id=<uuid>&action=login&page=1&page_size=50

# Less efficient: Paginating through all logs then filtering client-side
GET /api/audit/logs?page=1&page_size=1000
```

## Legacy Endpoints

Some endpoints use `page_size` instead of `limit` for historical reasons. These endpoints follow the same pagination pattern but use different parameter names:

### Endpoints Using page_size

- `GET /audit/logs` - Uses `page_size` (default: 50)
- `GET /execution-history` - Uses `page_size` (default: 50)

**Example:**
```bash
GET /api/audit/logs?page=1&page_size=50
```

**Response:**
```json
{
  "logs": [...],
  "page": 1,
  "page_size": 50,
  "total": 500,
  "total_pages": 10
}
```

**Note:** The response uses `page_size` instead of `limit`, but the behavior is identical.

## Paginated Endpoints Reference

| Endpoint | Parameter Name | Default Limit | Max Limit |
|----------|---------------|---------------|-----------|
| GET /flows | `limit` | 20 | 100 |
| GET /flows/:id/versions | `limit` | 20 | 100 |
| GET /executions | `limit` | 20 | 100 |
| GET /llm-configs | `limit` | 20 | 100 |
| GET /vector-configs | `limit` | 20 | 100 |
| GET /sessions | `limit` | 20 | 100 |
| GET /audit/logs | `page_size` | 50 | 100 |
| GET /execution-history | `page_size` | 50 | 100 |

## Best Practices

### 1. Always Check total_pages

Before implementing pagination, check if pagination is necessary:

```javascript
const response = await fetch('/api/flows?page=1&limit=20');
const data = await response.json();

if (data.total_pages === 1) {
  // All items fit on one page, no pagination needed
  displayItems(data.data);
} else {
  // Multiple pages, implement pagination UI
  displayItemsWithPagination(data);
}
```

### 2. Provide User Feedback

Always show users where they are in the pagination:

```javascript
function renderPaginationInfo(data) {
  return `Page ${data.page} of ${data.total_pages} (${data.total} total items)`;
}
```

### 3. Handle Empty Results Gracefully

```javascript
if (data.data.length === 0) {
  if (data.page > data.total_pages) {
    // User navigated beyond available pages
    showMessage("No more items available");
  } else {
    // No items match the query
    showMessage("No items found");
  }
}
```

### 4. Preserve Pagination State

When users navigate away and return, preserve their pagination state:

```javascript
// Save to URL query parameters
const url = new URL(window.location);
url.searchParams.set('page', currentPage);
url.searchParams.set('limit', currentLimit);
window.history.pushState({}, '', url);

// Or save to localStorage
localStorage.setItem('flowsPage', currentPage);
localStorage.setItem('flowsLimit', currentLimit);
```

### 5. Debounce Pagination Requests

When implementing features like infinite scroll, debounce requests to avoid overwhelming the server:

```javascript
let debounceTimer;

function debouncedLoadMore() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    loadMoreItems();
  }, 300);
}
```

## Testing Pagination

### Test Cases

When testing pagination, verify these scenarios:

1. **First page**: `?page=1&limit=20`
2. **Middle page**: `?page=5&limit=20`
3. **Last page**: `?page=8&limit=20`
4. **Beyond last page**: `?page=100&limit=20`
5. **Small limit**: `?page=1&limit=1`
6. **Large limit**: `?page=1&limit=100`
7. **Invalid limit (too small)**: `?page=1&limit=0`
8. **Invalid limit (too large)**: `?page=1&limit=150`
9. **With filters**: `?page=1&limit=20&provider=openai`
10. **Empty results**: Query that returns no items

### Example Test (JavaScript/Jest)

```javascript
describe('Pagination', () => {
  test('should return first page with default limit', async () => {
    const response = await fetch('/api/flows?page=1');
    const data = await response.json();
    
    expect(data.page).toBe(1);
    expect(data.limit).toBe(20);
    expect(data.data.length).toBeLessThanOrEqual(20);
    expect(data.total_pages).toBeGreaterThanOrEqual(1);
  });

  test('should return empty array for page beyond total', async () => {
    const response = await fetch('/api/flows?page=9999');
    const data = await response.json();
    
    expect(data.data).toEqual([]);
    expect(data.page).toBe(9999);
  });

  test('should reject invalid limit', async () => {
    const response = await fetch('/api/flows?page=1&limit=150');
    
    expect(response.status).toBe(400);
    const error = await response.json();
    expect(error.error).toContain('Limit cannot exceed 100');
  });
});
```

## Migration Notes

### For API Consumers

If you're migrating from an older version of the API:

1. **Page numbers now start from 1** (previously started from 0 in some endpoints)
   - Old: `?page=0` for first page
   - New: `?page=1` for first page

2. **Response format is now standardized**
   - All endpoints return `data`, `page`, `limit`, `total`, `total_pages`

3. **Limit constraints are enforced**
   - Maximum limit is now 100 (previously unlimited in some endpoints)

### For Internal Developers

The internal application layer uses 0-based pagination, but the API layer converts to 1-based:

```rust
// Handler layer (API) - 1-based
let page = query.page.unwrap_or(1).saturating_sub(1); // Convert to 0-based

// Application service layer - 0-based
let offset = page * limit;

// Response - Convert back to 1-based
let response = PaginatedResponse {
    page: page + 1,  // Convert back to 1-based
    // ...
};
```

## FAQ

### Q: Why does the API use 1-based pagination?

**A:** 1-based pagination is more intuitive for API consumers and matches common UI patterns (e.g., "Page 1 of 10"). Internally, the system uses 0-based pagination for consistency with programming conventions.

### Q: What happens if I request page 0?

**A:** Page 0 is treated as page 1. The API will return the first page of results.

### Q: Can I request all items without pagination?

**A:** No, all list endpoints require pagination. However, you can use the maximum limit of 100 and iterate through pages to retrieve all items.

### Q: Why is there a maximum limit of 100?

**A:** The limit prevents excessive memory usage and ensures consistent API performance. For larger datasets, iterate through multiple pages.

### Q: How do I know if there are more pages?

**A:** Compare `page` with `total_pages`. If `page < total_pages`, more pages are available.

### Q: What's the difference between limit and page_size?

**A:** They serve the same purpose. Some legacy endpoints use `page_size` for historical reasons, but new endpoints use `limit`. Both have the same constraints (min: 1, max: 100).

### Q: Can I change the default limit?

**A:** No, the default limit is fixed at 20 (or 50 for some endpoints). You must explicitly specify a different limit in each request.

### Q: Does pagination work with filtering?

**A:** Yes, pagination works seamlessly with all filtering parameters. Filters are applied first, then pagination is applied to the filtered results.

## Support

For questions or issues related to pagination:

1. Check this guide and the main API documentation
2. Review the examples in this document
3. Test your pagination logic with the provided test cases
4. Contact the development team if issues persist

## Changelog

### Version 2.0 (Current)
- Standardized all pagination endpoints to use consistent parameters
- Changed API to use 1-based page numbers (internal remains 0-based)
- Added `total_pages` to all responses
- Enforced limit constraints (min: 1, max: 100)
- Standardized response format across all endpoints

### Version 1.0 (Legacy)
- Mixed pagination approaches across endpoints
- Some endpoints used 0-based, others used 1-based
- Inconsistent parameter names (`page_size` vs `limit`)
- No enforced limit constraints

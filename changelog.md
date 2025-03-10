# Changelog

## 1.2.0 (2025-03-10)

### Breaking Changes

- **Required Document ID for Updates**: The `doc_id` parameter is now required for all update operations. The ability to update the most recent record without specifying a `doc_id` has been removed for consistency and clarity.

### New Features

- **Document IDs**: Each record now automatically receives a unique `doc_id` (UUID) when created
- **Targeted Record Updates**: Added ability to update specific records by their `doc_id` instead of only the most recent record
- **Improved Search Capabilities**: Enhanced search functionality to find records by `doc_id` in both simple and advanced searches
- **Better Update Response**: When updating a record by `doc_id`, the API now returns the updated record instead of the latest record
- **Direct Document Access**: Added new API endpoint `/table/<table_name>/doc/<doc_id>` to directly retrieve a specific document by its `doc_id`

### Security Improvements

- **Protected Document IDs**: Added protection to ensure that `doc_id` values cannot be set or modified by users. Any attempt to include a `doc_id` in the data when creating or updating records will be ignored.

### Bug Fixes

- **Persist Operation**: Fixed an issue where the `persist` operation incorrectly required a `doc_id`. Created a separate request structure to ensure `persist` operations don't require a `doc_id` parameter.

### Documentation

- Added detailed explanation about the complete replacement behavior during updates (no property merging)
- Updated API examples to demonstrate searching and updating records by `doc_id`
- Removed examples and documentation for updating the latest record without a `doc_id`

## 1.1.0 (2025-03-06)

- **List All Tables Feature**: Now it's possible to list all tables present for a specific Database.

## 1.0.0 (2025-03-04)

### Main Changes Compared to the v0.0.3

- **Enhanced Security**: Implementation of AES-256-GCM encryption for all stored data, replacing the previous model without encryption
- **Password Management**: Password-based key derivation system using SHA-256, offering greater security for data access
- **Advanced Search Capabilities**: Implementation of powerful search functionality with multiple comparison operators (equals, greater than, contains, starts with, etc.)
- **Real-Time Event System**: Addition of WebSocket-based event system for real-time notifications when data changes
- **Automatic Backup**: Addition of automatic backup system during critical operations such as password changes
- **Disk Space Verification**: Implementation of system resource checks before operations that require additional space
- **Operation Atomicity**: Guarantee that critical operations are atomic (all or nothing) with rollback mechanism in case of failure
- **Complete REST API**: Redesign of the API to offer more functionality and better integration with applications
- **Optimized File Structure**: Reorganization of the file structure for better performance and management
- **Data Segmentation**: Implementation of efficient data segmentation in blocks of 1000 records
- **Multiple Database Support**: Ability to manage multiple independent databases
- **Focus on Data History**: Maintains the concept of history tracking from the previous version, but with more robust implementation
- **Paradigm Shift**: Transition from a blockchain-inspired model to a more traditional and efficient history database system

### Removals

- Elimination of the "units" concept and transfers present in the previous version
- Removal of the blockchain-style chained blocks approach
- Simplification of the user model

### Technical Improvements

- Complete rewrite of the codebase for greater efficiency and maintainability
- Better error handling and failure recovery
- More comprehensive documentation and usage examples
- WebSocket API for real-time data synchronization
- Flexible query system with support for complex search criteria

# Changelog

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

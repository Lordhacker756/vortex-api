# Vortex API - Real-time Polling System

A modern, real-time polling system built with Rust using Axum framework and MongoDB. Features include real-time vote tracking, secure authentication, and comprehensive API documentation.

## 🚀 Features

- **Real-time Vote Tracking**: Live updates using Server-Sent Events (SSE)
- **Secure Authentication**: Email-based verification system
- **MongoDB Integration**: Robust data persistence
- **CORS Support**: Configured for cross-origin requests
- **Session Management**: Secure user sessions
- **Error Handling**: Comprehensive error handling system
- **Logging**: Detailed request/response logging

## 🔧 Best Practices

- **Clean Architecture**
  - Repository Pattern for data access
  - DTO Pattern for request/response handling
  - Middleware for cross-cutting concerns
  - Centralized error handling

- **Security**
  - Session-based authentication
  - Email verification
  - Protected routes
  - CORS configuration

- **Performance** 
  - Async/await patterns
  - Connection pooling
  - Efficient data structures

## 📚 API Documentation

### Authentication Routes

#### 1. Initiate Registration
- **POST** `/auth/register`
- **Query Parameters**: `{ "username": "string" }`
- **Response**: Webauthn registration challenge

#### 2. Complete Registration
- **POST** `/auth/register/complete`
- **Body**: Webauthn credential response
- **Response**: `{ "status": 200, "message": "Registration completed successfully" }`

#### 3. Initiate Login
- **POST** `/auth/login`
- **Query Parameters**: `{ "username": "string" }`
- **Response**: Webauthn authentication challenge

#### 4. Complete Login
- **POST** `/auth/login/complete`
- **Body**: Webauthn authentication response
- **Response**: `{ "status": 200, "message": "Login successful", "user_id": "string", "timestamp": "string" }`

### Poll Routes

#### 1. Get All Polls
- **GET** `/polls`
- **Response**: List of all public polls

#### 2. Get User's Polls
- **GET** `/polls/manage`
- **Auth**: Required
- **Response**: List of polls created by the authenticated user

#### 3. Create Poll
- **POST** `/polls`
- **Body**: 
```json
{
    "title": "string",
    "description": "string",
    "options": [
        { "text": "string" }
    ],
    "settings": {
        "is_private": boolean,
        "multiple_choice": boolean
    }
}
```

#### 4. Get Poll
- **GET** `/polls/:poll_id`
- **Response**: Poll details

#### 5. Update Poll
- **PATCH** `/polls/:poll_id`
- **Body**: 
```json
{
    "title": "string",
    "description": "string",
    "options": [
        { "text": "string" }
    ]
}
```

#### 6. Cast Vote
- **GET** `/polls/:poll_id/vote`
- **Query Parameters**: `{ "optionId": "string" }`
- **Auth**: Required

#### 7. Check Vote Eligibility
- **GET** `/polls/:poll_id/can-vote`
- **Auth**: Required
- **Response**: `{ "data": boolean }`

#### 8. Get Poll Results
- **GET** `/polls/:poll_id/results`
- **Query Parameters**: `{ "live": boolean }`
- **Response**: 
  - If `live=true`: Server-Sent Events stream of real-time results
  - If `live=false`: Current poll results

#### 9. Close Poll
- **GET** `/polls/:poll_id/close`
- **Auth**: Required (Poll owner only)

#### 10. Reset Poll
- **GET** `/polls/:poll_id/reset`
- **Auth**: Required (Poll owner only)

## 🛠️ Setup & Installation

1. **Prerequisites**
   - Rust (latest stable)
   - MongoDB

2. **Environment Setup**
   ```bash
   cp .env.example .env
   # Configure your environment variables
   ```

3. **Build & Run**
   ```bash
   cargo build --release
   cargo run
   ```

## 🧪 Testing

```bash
cargo test
```

## 📜 License

MIT License - See LICENSE file for details
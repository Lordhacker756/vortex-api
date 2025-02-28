# 🌪️ Vortex API - Real-time Polling System ⚡️

## 📌 About
🗳️ **Vortex** is a lightning-fast polling API that lets you create, manage, and analyze polls in real-time. Built with Rust for performance, secured with WebAuthn, and powered by MongoDB. Perfect for live events, classroom engagement, or gathering quick feedback! 🚀

## 🚀 Features

- **🔄 Real-time Vote Tracking**: Live updates using Server-Sent Events (SSE)
- **🔐 Secure Authentication**: Passwordless WebAuthn authentication
- **🗃️ MongoDB Integration**: Robust data persistence
- **🌐 CORS Support**: Configured for cross-origin requests
- **🔑 JWT Authentication**: Secure token-based authentication
- **⚠️ Error Handling**: Comprehensive error handling system
- **📊 Logging**: Detailed request/response logging

## 🔧 Best Practices

- **🏗️ Clean Architecture**
  - 📚 Repository Pattern for data access
  - 🔄 DTO Pattern for request/response handling
  - 🔗 Middleware for cross-cutting concerns
  - 🎯 Centralized error handling

- **🔒 Security**
  - 👆 WebAuthn passwordless authentication
  - 🎟️ JWT for session management
  - 🛡️ Protected routes
  - 🌍 CORS configuration

- **⚡ Performance** 
  - ⏱️ Async/await patterns
  - 🔌 Connection pooling
  - 📈 Efficient data structures

## 📚 API Documentation

### 🔐 Authentication Routes

#### 1. 📝 Initiate Registration
- **GET** `/auth/register`
- **Query Parameters**: `{ "username": "string" }`
- **Response**: WebAuthn registration challenge

#### 2. ✅ Complete Registration
- **POST** `/auth/verify-register/{username}`
- **Body**: WebAuthn credential response
- **Response**: `{ "status": 200, "message": "Registration completed successfully", "token": "jwt-token" }`

#### 3. 🔑 Initiate Login
- **GET** `/auth/login`
- **Query Parameters**: `{ "username": "string" }`
- **Response**: WebAuthn authentication challenge

#### 4. 🔓 Complete Login
- **POST** `/auth/verify-login/{username}`
- **Body**: WebAuthn authentication response
- **Response**: `{ "status": 200, "message": "Login successful", "user_id": "string", "token": "jwt-token", "timestamp": "string" }`

#### 5. 👋 Logout
- **POST** `/auth/logout`
- **Response**: `{ "status": 200, "message": "Logged out successfully" }`

### 📊 Poll Routes

#### 1. 📋 Get All Polls
- **GET** `/polls`
- **Response**: List of all public polls
```json
{
  "status": 200,
  "message": "All posts fetched successfully",
  "data": [
    {
      "id": "string",
      "title": "string",
      "description": "string",
      "options": [
        {
          "id": "string",
          "text": "string",
          "votes": 0
        }
      ],
      "createdBy": "string",
      "createdAt": "string",
      "settings": {
        "isPrivate": false,
        "multipleChoice": false
      },
      "status": "active"
    }
  ],
  "timestamp": "string",
  "error": null
}
```

#### 2. 🧑‍💼 Get User's Polls
- **GET** `/polls/manage`
- **Auth**: Required (Bearer token)
- **Response**: List of polls created by the authenticated user

#### 3. ➕ Create Poll
- **POST** `/polls`
- **Auth**: Required (Bearer token)
- **Body**: 
```json
{
  "title": "string",
  "description": "string",
  "options": [
    { "text": "string" }
  ],
  "settings": {
    "isPrivate": false,
    "multipleChoice": false
  }
}
```

#### 4. 🔍 Get Poll
- **GET** `/polls/{poll_id}`
- **Response**: Poll details

#### 5. 🔄 Update Poll
- **PATCH** `/polls/{poll_id}`
- **Auth**: Required (Bearer token, poll owner only)
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

#### 6. 🗳️ Cast Vote
- **GET** `/polls/{poll_id}/vote`
- **Query Parameters**: `{ "optionId": "string" }`
- **Auth**: Required (Bearer token)

#### 7. ✓ Check Vote Eligibility
- **GET** `/polls/{poll_id}/can-vote`
- **Auth**: Required (Bearer token)
- **Response**: Boolean indicating if user can vote

#### 8. 📈 Get Poll Results
- **GET** `/polls/{poll_id}/results`
- **Query Parameters**: `{ "live": boolean }`
- **Response**: 
  - If `live=true`: Server-Sent Events stream of real-time results
  - If `live=false`: Current poll results

#### 9. 🚫 Close Poll
- **GET** `/polls/{poll_id}/close`
- **Auth**: Required (Bearer token, poll owner only)
- **Response**: Updated poll with status "closed"

#### 10. 🔄 Reset Poll
- **GET** `/polls/{poll_id}/reset`
- **Auth**: Required (Bearer token, poll owner only)
- **Response**: Updated poll with reset votes

## 🛠️ Setup & Installation

1. **📋 Prerequisites**
   - 🦀 Rust (latest stable)
   - 🍃 MongoDB

2. **⚙️ Environment Setup**
   ```bash
   cp .env.example .env
   # Configure your environment variables including JWT_SECRET
   ```

3. **🏗️ Build & Run**
   ```bash
   cargo build --release
   cargo run
   ```


## 📜 License

MIT License - See LICENSE file for details

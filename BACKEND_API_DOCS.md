# Meet Me in the Middle - Backend API Documentation

## 🚀 Overview

**Meet Me in the Middle** is a Rust-based backend service that helps groups find optimal meeting venues. It supports both instant recommendations and collaborative session-based planning where multiple users can join and contribute their locations dynamically.

### 🏗️ Technology Stack

- **Language**: Rust
- **Framework**: Axum (async web framework)
- **APIs**: Google Maps Platform (Places API New, Distance Matrix API, Geocoding API)
- **Data Format**: JSON over HTTPS/REST
- **Session Storage**: In-memory concurrent HashMap with RwLock
- **Deployment**: Standalone binary, Docker-ready

### 🎯 Core Features

- **Instant Recommendations**: Single API call for quick venue suggestions
- **Collaborative Sessions**: Multi-user planning with real-time updates
- **Smart Location Input**: Supports both GPS coordinates and address strings
- **Multi-Modal Transport**: Drive, Walk, Transit, Bicycle routing
- **Intelligent Ranking**: Fairness-based algorithm minimizing travel time imbalances
- **Google Maps Integration**: Direct links to venues and navigation

---

## 📊 Data Models

### Core Types

```typescript
// Session Status Flow
type SessionStatus =
	| "waiting_for_participants"
	| "ready_for_recommendations"
	| "generating_recommendations"
	| "recommendations_ready"
	| "expired";

// Transport Modes (case-sensitive)
type TransportMode = "DRIVE" | "WALK" | "TRANSIT" | "BICYCLE";

// Location Input (flexible)
type ParticipantInput =
	| { lat: number; lng: number } // GPS coordinates
	| { address: string }; // Address string

// Price Levels (Google Places API format)
type PriceLevel =
	| "PRICE_LEVEL_FREE"
	| "PRICE_LEVEL_INEXPENSIVE"
	| "PRICE_LEVEL_MODERATE"
	| "PRICE_LEVEL_EXPENSIVE"
	| "PRICE_LEVEL_VERY_EXPENSIVE";

// Venue Categories (Google Places API standard types)
type PlaceCategory =
	| "restaurant"
	| "cafe"
	| "bar"
	| "bakery"
	| "meal_takeaway"
	| "meal_delivery"
	| "movie_theater"
	| "museum"
	| "art_gallery"
	| "night_club"
	| "park"
	| "zoo"
	| "stadium"
	| "gym"
	| "shopping_mall"
	| "store"
	| "book_store"
	| "clothing_store"
	| "library"
	| "tourist_attraction";
```

### Session Models

```typescript
interface Session {
	id: string; // UUID
	name: string; // User-friendly session name
	creator_id: string; // Creator's user ID
	participants: Record<string, Participant>; // user_id -> Participant
	settings: SessionSettings;
	status: SessionStatus;
	created_at: string; // ISO 8601 timestamp
	updated_at: string; // ISO 8601 timestamp
	expires_at: string; // ISO 8601 timestamp
}

interface Participant {
	user_id: string; // UUID
	name: string; // Display name
	location: ParticipantInput | null;
	joined_at: string; // ISO 8601 timestamp
	is_ready: boolean; // Has confirmed location
}

interface SessionSettings {
	categories: PlaceCategory[]; // Venue types to search
	transport_mode: TransportMode;
	limit: number; // Max recommendations to return
	auto_refresh: boolean; // Auto-generate when participants join
	require_all_participants: boolean; // Wait for everyone to be ready
}
```

### Recommendation Models

```typescript
interface EnhancedVenueRecommendation {
	name: string; // Venue name
	address: string; // Full formatted address
	lat: number; // Latitude
	lng: number; // Longitude
	rating?: number; // Google rating (0-5)
	reviews?: number; // Number of reviews
	google_maps_url: string; // Direct Google Maps link
	travel_times: TravelTimeInfo[]; // Per-participant travel info
	category: string; // Primary place type
	price_level?: PriceLevel; // Cost indicator
}

interface TravelTimeInfo {
	participant_index: number; // Index in original participants array
	travel_time_minutes: number; // Travel time in minutes
	transport_mode: string; // Mode used for this calculation
	google_maps_directions_url: string; // Direct navigation link
}
```

---

## 🌐 API Endpoints

### Health & Status

#### `GET /health`

**Purpose**: Service health check

```json
// Response
{
	"status": "healthy",
	"service": "mmitm",
	"version": "0.1.0"
}
```

---

### 🎯 Instant Recommendations

#### `POST /recommendations`

**Purpose**: Get venue recommendations without creating a session (quick one-time use)

```json
// Request Body
{
  "participants": [
    { "lat": 12.9716, "lng": 77.5946 },      // GPS coordinates
    { "address": "Bangalore, Karnataka" }     // Address string
  ],
  "categories": ["restaurant", "cafe"],
  "transport_mode": "DRIVE",                  // DRIVE|WALK|TRANSIT|BICYCLE
  "limit": 10                                 // Optional, default: 10
}

// Response: Array of EnhancedVenueRecommendation
[
  {
    "name": "The French Press",
    "address": "123 Brigade Road, Bangalore, Karnataka 560001, India",
    "lat": 12.9745,
    "lng": 77.6096,
    "rating": 4.3,
    "reviews": 1250,
    "google_maps_url": "https://maps.google.com/?cid=1234567890",
    "travel_times": [
      {
        "participant_index": 0,
        "travel_time_minutes": 12,
        "transport_mode": "DRIVE",
        "google_maps_directions_url": "https://maps.google.com/dir/..."
      },
      {
        "participant_index": 1,
        "travel_time_minutes": 15,
        "transport_mode": "DRIVE",
        "google_maps_directions_url": "https://maps.google.com/dir/..."
      }
    ],
    "category": "restaurant",
    "price_level": "PRICE_LEVEL_MODERATE"
  }
]
```

---

### 👥 Collaborative Sessions

#### `POST /sessions`

**Purpose**: Create a new collaborative planning session

```json
// Request Body
{
  "name": "Friday Team Dinner",
  "creator_name": "John Doe",
  "settings": {
    "categories": ["restaurant", "bar"],
    "transport_mode": "DRIVE",
    "limit": 15,
    "auto_refresh": true,
    "require_all_participants": false
  }
}

// Response
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "join_code": "ABC123",                    // Short code for easy sharing
  "session": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Friday Team Dinner",
    "creator_id": "creator-user-id",
    "participants": {
      "creator-user-id": {
        "user_id": "creator-user-id",
        "name": "John Doe",
        "location": null,
        "joined_at": "2025-08-14T10:30:00Z",
        "is_ready": false
      }
    },
    "settings": { /* same as request */ },
    "status": "waiting_for_participants",
    "created_at": "2025-08-14T10:30:00Z",
    "updated_at": "2025-08-14T10:30:00Z",
    "expires_at": "2025-08-15T10:30:00Z"    // 24h expiry
  }
}
```

#### `POST /sessions/join`

**Purpose**: Join an existing session using join code

```json
// Request Body
{
  "join_code": "ABC123",                    // Either join_code or session_id
  "participant_name": "Jane Smith"
}

// Response: Same format as create session
{
  "user_id": "new-participant-user-id",
  "session": { /* updated session object */ }
}
```

#### `PUT /sessions/{session_id}/participants/{user_id}/location`

**Purpose**: Update participant's location and ready status

```json
// Request Body
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "participant-user-id",
  "location": { "lat": 12.9716, "lng": 77.5946 },  // or {"address": "..."}
  "is_ready": true
}

// Response: SessionStatusResponse
{
  "session": { /* updated session object */ },
  "recommendations": null                   // or array if available
}
```

#### `POST /sessions/{session_id}/recommendations`

**Purpose**: Generate venue recommendations for session participants

```json
// Request Body
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "requesting-user-id"          // Must be creator or participant
}

// Response: SessionStatusResponse with recommendations
{
  "session": {
    /* session object with status: "recommendations_ready" */
  },
  "recommendations": [
    /* Array of EnhancedVenueRecommendation */
  ]
}
```

#### `GET /sessions/{session_id}`

**Purpose**: Get full session status (for detailed updates)

```json
// Response: SessionStatusResponse
{
	"session": {
		/* complete session object */
	},
	"recommendations": [
		/* recommendations if available */
	]
}
```

#### `GET /sessions/{session_id}/health`

**Purpose**: Lightweight polling endpoint for real-time updates

```json
// Response: Minimal session health info
{
	"session_id": "550e8400-e29b-41d4-a716-446655440000",
	"status": "waiting_for_participants",
	"participant_count": 3,
	"ready_count": 2, // Participants with confirmed locations
	"updated_at": "2025-08-14T10:35:00Z"
}
```

#### `POST /sessions/cleanup`

**Purpose**: Admin endpoint to clean up expired sessions

```json
// Response
{
	"message": "Expired sessions cleaned up",
	"timestamp": "2025-08-14T11:00:00Z"
}
```

---

## 🎯 Ranking Algorithm

The backend uses a sophisticated fairness-based ranking algorithm:

1. **Primary Factor**: Fairness Score (lower is better)

   - Minimizes the maximum travel time difference between participants
   - Penalizes venues where some participants travel much longer than others

2. **Secondary Factor**: Total Travel Time

   - Sum of all participants' travel times
   - Optimizes for overall efficiency

3. **Penalty System**:
   - Two-participant sessions with >10 minute imbalance get penalized
   - Ensures reasonable fairness even with just two people

**Example**: If Alice takes 10 minutes and Bob takes 30 minutes to reach a venue, the fairness score reflects this 20-minute imbalance.

---

## ⚠️ Error Handling

All endpoints return consistent error format:

```json
// Error Response Format
{
	"error": "Descriptive error message",
	"status": 400 // HTTP status code
}

// Common Error Scenarios:
// 400 - Bad Request: Invalid input, missing fields, malformed JSON
// 400 - Session not found, invalid join code
// 500 - Google API quota exceeded, external service errors
// 500 - Internal server errors, geocoding failures
```

---

## 🔧 Configuration

### Environment Variables Required:

- `GOOGLE_MAPS_API_KEY`: Google Cloud Platform API key with enabled services:
  - Places API (New)
  - Distance Matrix API
  - Geocoding API
- `PORT`: Server port (optional, default: 3000)

### Google API Requirements:

- **Places API (New)**: For venue discovery with enhanced data
- **Distance Matrix API**: For travel time calculations
- **Geocoding API**: For address-to-coordinate conversion

---

## 📱 React Native Integration Guide

### Essential Libraries Needed:

```bash
npm install axios                          # HTTP client
npm install @react-native-community/geolocation  # GPS location
npm install react-native-maps             # Map display
npm install @react-navigation/native      # Screen navigation
npm install @reduxjs/toolkit react-redux  # State management
```

### Key Integration Patterns:

#### 1. Location Input Component

```typescript
// Support both GPS and address input
interface LocationInputProps {
	onLocationSelected: (location: ParticipantInput) => void;
}

// Allow users to:
// - Use current GPS location
// - Search and select from address suggestions
// - Manually enter coordinates
```

#### 2. Real-time Session Updates

```typescript
// Polling strategy for session updates
const useSessionPolling = (sessionId: string) => {
	useEffect(() => {
		const interval = setInterval(async () => {
			try {
				// Use lightweight health endpoint for frequent checks
				const health = await api.get(`/sessions/${sessionId}/health`);

				// Only fetch full session if meaningful changes detected
				if (health.updated_at > lastKnownUpdate) {
					const fullSession = await api.get(`/sessions/${sessionId}`);
					updateSessionState(fullSession);
				}
			} catch (error) {
				handlePollingError(error);
			}
		}, 3000); // Poll every 3 seconds

		return () => clearInterval(interval);
	}, [sessionId]);
};
```

#### 3. Recommendation Display

```typescript
// Display venues with travel time visualization
interface VenueCardProps {
	venue: EnhancedVenueRecommendation;
	participants: Participant[];
}

// Features to implement:
// - Show travel times for each participant
// - "Open in Google Maps" button using google_maps_url
// - "Get Directions" button using google_maps_directions_url
// - Rating display with star visualization
// - Price level indicator
```

#### 4. Session Flow Management

```typescript
// Recommended screen flow:
// 1. HomeScreen: Choose instant vs session mode
// 2. CreateSessionScreen: Setup session with categories/transport
// 3. JoinSessionScreen: Enter join code
// 4. SessionLobbyScreen: Show participants, location status
// 5. LocationInputScreen: GPS or address input
// 6. RecommendationsScreen: Display venue results
// 7. VenueDetailScreen: Individual venue information
```

### Sample API Integration:

```typescript
// API Client Setup
const API_BASE = "https://your-backend-url.com";

class MMITMApi {
	// Create session
	async createSession(
		name: string,
		creatorName: string,
		settings: SessionSettings
	) {
		const response = await fetch(`${API_BASE}/sessions`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ name, creator_name: creatorName, settings }),
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error);
		}

		return response.json();
	}

	// Join session
	async joinSession(joinCode: string, participantName: string) {
		const response = await fetch(`${API_BASE}/sessions/join`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				join_code: joinCode,
				participant_name: participantName,
			}),
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error);
		}

		return response.json();
	}

	// Update location
	async updateLocation(
		sessionId: string,
		userId: string,
		location: ParticipantInput,
		isReady: boolean = true
	) {
		const response = await fetch(
			`${API_BASE}/sessions/${sessionId}/participants/${userId}/location`,
			{
				method: "PUT",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					session_id: sessionId,
					user_id: userId,
					location,
					is_ready: isReady,
				}),
			}
		);

		return response.json();
	}

	// Get instant recommendations
	async getRecommendations(
		participants: ParticipantInput[],
		categories: string[],
		transportMode: TransportMode = "DRIVE",
		limit: number = 10
	) {
		const response = await fetch(`${API_BASE}/recommendations`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				participants,
				categories,
				transport_mode: transportMode,
				limit,
			}),
		});

		return response.json();
	}
}
```

### State Management Pattern:

```typescript
// Redux store slices for session management
interface SessionState {
	currentSession: Session | null;
	currentUserId: string | null;
	recommendations: EnhancedVenueRecommendation[];
	isLoading: boolean;
	error: string | null;
}

// Key actions:
// - createSession
// - joinSession
// - updateParticipantLocation
// - generateRecommendations
// - pollSessionStatus
```

This backend provides a complete foundation for building a collaborative, real-time location-based meeting planner with both instant and session-based workflows.

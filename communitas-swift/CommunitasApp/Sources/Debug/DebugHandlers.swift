import Foundation
import SwiftUI
import CommunitasKit

/// Communitas-specific debug action handlers
/// Registers handlers with DebugServer for programmatic app control
@MainActor
public struct DebugHandlers {

    /// Register all Communitas debug handlers
    public static func register(appState: AppState) {
        let server = DebugServer.shared

        // MARK: - State Query Handlers

        server.registerHandler("state") { _ in
            let state: [String: Any] = [
                "fourWords": appState.fourWords,
                "displayName": appState.displayName,
                "isInitialized": appState.isInitialized,
                "isNetworking": appState.isNetworking,
                "isAuthenticated": appState.isAuthenticated,
                "contactCount": appState.contacts.count,
                "entityCount": appState.entities.count,
                "bootstrapAddress": appState.bootstrapAddress,
                "errorMessage": appState.errorMessage ?? NSNull()
            ]
            return try JSONSerialization.data(withJSONObject: state, options: [.prettyPrinted])
        }

        server.registerHandler("contacts") { _ in
            let contactList = appState.contacts.map { contact -> [String: Any] in
                var contactDict: [String: Any] = [
                    "id": contact.id,
                    "fourWords": contact.fourWords as Any,
                    "displayName": contact.displayName ?? NSNull(),
                    "isFavourite": contact.isFavourite,
                    "isOnline": contact.isOnline,
                    "isLocalOnly": contact.isLocalOnly
                ]
                // Include endpoint tracking fields
                contactDict["lastSeenEndpoint"] = contact.lastSeenEndpoint ?? NSNull()
                contactDict["endpointUpdatedAt"] = contact.endpointUpdatedAt?.timeIntervalSince1970 ?? NSNull()
                contactDict["endpointSuccessCount"] = contact.endpointSuccessCount
                contactDict["endpointFailureCount"] = contact.endpointFailureCount
                return contactDict
            }
            return try JSONSerialization.data(withJSONObject: ["contacts": contactList, "count": contactList.count], options: [.prettyPrinted])
        }

        // MARK: - Contact Endpoint Tracking Handlers

        server.registerHandler("contactEndpoint") { body in
            // Get endpoint info for a specific contact
            // Expects: { "fourWords": "word-word-word-word" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            // Find the contact
            guard let contact = appState.contacts.first(where: { $0.fourWords == fourWords }) else {
                throw DebugError.notFound("Contact with fourWords '\(fourWords)' not found")
            }

            // Also get endpoint from client if initialized
            var clientEndpoint: String? = nil
            if appState.isInitialized, let client = appState.client {
                clientEndpoint = client.gossipContactGetEndpoint(fourWords: fourWords)
            }

            return try JSONSerialization.data(withJSONObject: [
                "fourWords": fourWords,
                "lastSeenEndpoint": contact.lastSeenEndpoint ?? NSNull(),
                "endpointUpdatedAt": (contact.endpointUpdatedAt?.timeIntervalSince1970 ?? NSNull()) as Any,
                "endpointSuccessCount": contact.endpointSuccessCount,
                "endpointFailureCount": contact.endpointFailureCount,
                "clientEndpoint": clientEndpoint ?? NSNull()
            ], options: [.prettyPrinted])
        }

        server.registerHandler("updateContactEndpoint") { body in
            // Update a contact's endpoint (four-word encoded)
            // Expects: { "fourWords": "word-word-word-word", "endpoint": "four-word-encoded-endpoint" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String,
                  let endpoint = json["endpoint"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' or 'endpoint' parameter")
            }

            guard appState.isInitialized, let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized")
            }

            do {
                try client.gossipContactUpdateEndpoint(fourWords: fourWords, endpoint: endpoint)

                // Update local contact state
                if let index = appState.contacts.firstIndex(where: { $0.fourWords == fourWords }) {
                    appState.contacts[index].lastSeenEndpoint = endpoint
                    appState.contacts[index].endpointUpdatedAt = Date()
                }

                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "fourWords": fourWords,
                    "endpoint": endpoint
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to update endpoint: \(error.localizedDescription)")
            }
        }

        server.registerHandler("recordConnectionSuccess") { body in
            // Record a successful connection to a contact's endpoint
            // Expects: { "fourWords": "word-word-word-word", "endpoint": "four-word-encoded-endpoint" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String,
                  let endpoint = json["endpoint"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' or 'endpoint' parameter")
            }

            guard appState.isInitialized, let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized")
            }

            do {
                try client.gossipContactRecordSuccess(fourWords: fourWords, endpoint: endpoint)

                // Update local contact state
                if let index = appState.contacts.firstIndex(where: { $0.fourWords == fourWords }) {
                    appState.contacts[index].endpointSuccessCount += 1
                    appState.contacts[index].lastSeenEndpoint = endpoint
                    appState.contacts[index].endpointUpdatedAt = Date()
                }

                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "fourWords": fourWords,
                    "endpoint": endpoint,
                    "message": "Connection success recorded"
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to record success: \(error.localizedDescription)")
            }
        }

        server.registerHandler("recordConnectionFailure") { body in
            // Record a failed connection attempt
            // Expects: { "fourWords": "word-word-word-word" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            guard appState.isInitialized, let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized")
            }

            do {
                try client.gossipContactRecordFailure(fourWords: fourWords)

                // Update local contact state
                if let index = appState.contacts.firstIndex(where: { $0.fourWords == fourWords }) {
                    appState.contacts[index].endpointFailureCount += 1
                }

                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "fourWords": fourWords,
                    "message": "Connection failure recorded"
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to record failure: \(error.localizedDescription)")
            }
        }

        server.registerHandler("entities") { _ in
            let entityList = appState.entities.map { entity -> [String: Any] in
                return [
                    "id": entity.id,
                    "name": entity.name,
                    "entityType": "\(entity.entityType)",
                    "description": entity.description ?? NSNull(),
                    "parentOrgId": entity.parentOrgId ?? NSNull()
                ]
            }
            return try JSONSerialization.data(withJSONObject: ["entities": entityList, "count": entityList.count], options: [.prettyPrinted])
        }

        server.registerHandler("messages") { body in
            // Expects: { "fourWords": "word-word-word-word" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            let messages = appState.getDirectMessages(for: fourWords)
            let messageList = messages.map { msg -> [String: Any] in
                return [
                    "id": msg.id,
                    "text": msg.text,
                    "author": msg.author,
                    "createdAt": msg.createdAt  // Int64 timestamp
                ]
            }
            return try JSONSerialization.data(withJSONObject: ["messages": messageList, "count": messageList.count], options: [.prettyPrinted])
        }

        // MARK: - Initialization Handlers

        server.registerHandler("setIdentity") { body in
            // Expects: { "fourWords": "word-word-word-word", "displayName": "Name" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String,
                  let displayName = json["displayName"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' or 'displayName' parameter")
            }

            // Set the identity properties
            appState.fourWords = fourWords
            appState.displayName = displayName

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "fourWords": fourWords,
                "displayName": displayName
            ], options: [])
        }

        server.registerHandler("initialize") { _ in
            appState.initialize()

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "isInitialized": appState.isInitialized,
                "fourWords": appState.fourWords
            ], options: [])
        }

        server.registerHandler("debugLogin") { body in
            // DEBUG-only: Skip passkey authentication for testing
            // Expects: { "fourWords": "word-word-word-word", "displayName": "Name" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String,
                  let displayName = json["displayName"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' or 'displayName' parameter")
            }

            // Initialize the client with provided credentials
            appState.initializeClientWithCredentials(fourWords: fourWords, displayName: displayName)

            // Set authenticated state
            appState.fourWords = fourWords
            appState.displayName = displayName
            appState.isInitialized = true
            appState.isAuthenticated = true

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "fourWords": fourWords,
                "displayName": displayName,
                "isAuthenticated": appState.isAuthenticated,
                "isInitialized": appState.isInitialized
            ], options: [])
        }

        // MARK: - Networking Handlers

        server.registerHandler("startNetworking") { body in
            var port: UInt16? = nil
            if let body = body,
               let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
               let portNum = json["port"] as? Int {
                port = UInt16(portNum)
            }

            appState.startNetworking(port: port)

            let portValue: Any = port.map { Int($0) } ?? NSNull()
            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "isNetworking": appState.isNetworking,
                "port": portValue
            ], options: [])
        }

        server.registerHandler("stopNetworking") { _ in
            appState.stopNetworking()

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "isNetworking": appState.isNetworking
            ], options: [])
        }

        server.registerHandler("setBootstrap") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let address = json["address"] as? String else {
                throw DebugError.invalidRequest("Missing 'address' parameter")
            }

            appState.bootstrapAddress = address

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "bootstrapAddress": address
            ], options: [])
        }

        server.registerHandler("connectToPeer") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            appState.connectToPeer(fourWords: fourWords)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "fourWords": fourWords
            ], options: [])
        }

        server.registerHandler("dialAddress") { body in
            // Dial a specific socket address to establish a peer connection
            // Expects: { "address": "127.0.0.1:49152" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let address = json["address"] as? String else {
                throw DebugError.invalidRequest("Missing 'address' parameter")
            }

            appState.dialAddress(address)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "address": address
            ], options: [])
        }

        server.registerHandler("networkInfo") { _ in
            // Get comprehensive network information
            guard let info = appState.getNetworkInfo() else {
                return try JSONSerialization.data(withJSONObject: [
                    "isActive": false,
                    "connectionIdentity": NSNull(),
                    "listenAddress": NSNull(),
                    "port": NSNull(),
                    "fourWords": appState.fourWords,
                    "isLocalOnlyMode": true
                ], options: [.prettyPrinted])
            }

            let result: [String: Any] = [
                "isActive": info.isActive,
                "connectionIdentity": info.connectionIdentity ?? NSNull(),
                "listenAddress": info.listenAddress ?? NSNull(),
                "port": info.port.map { Int($0) } ?? NSNull(),
                "fourWords": info.fourWords,
                "isLocalOnlyMode": info.isLocalOnlyMode
            ]
            return try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])
        }

        // MARK: - Contact Handlers

        server.registerHandler("addContact") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            let displayName = json["displayName"] as? String
            appState.addContact(fourWords: fourWords, displayName: displayName)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "fourWords": fourWords,
                "contactCount": appState.contacts.count
            ], options: [])
        }

        server.registerHandler("removeContact") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            // Check if contact exists before removing
            if appState.contacts.contains(where: { $0.fourWords == fourWords }) {
                appState.removeContact(fourWords: fourWords)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "fourWords": fourWords,
                    "contactCount": appState.contacts.count
                ], options: [])
            } else {
                throw DebugError.notFound("Contact not found: \(fourWords)")
            }
        }

        // MARK: - Messaging Handlers

        server.registerHandler("sendMessage") { body in
            // Debug logging
            if let body = body {
                print("[sendMessage] Body size: \(body.count) bytes")
                if let bodyString = String(data: body, encoding: .utf8) {
                    print("[sendMessage] Body: '\(bodyString)'")
                }
            } else {
                print("[sendMessage] Body is nil!")
            }

            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let toFourWords = json["to"] as? String,
                  let text = json["text"] as? String else {
                if let body = body, let bodyString = String(data: body, encoding: .utf8) {
                    print("[sendMessage] JSON parse failed for: '\(bodyString)'")
                }
                throw DebugError.invalidRequest("Missing 'to' or 'text' parameter")
            }

            appState.sendDirectMessage(to: toFourWords, text: text)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "to": toFourWords,
                "text": text
            ], options: [])
        }

        server.registerHandler("loadMessages") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let fourWords = json["fourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'fourWords' parameter")
            }

            appState.loadDirectMessages(for: fourWords)
            let messages = appState.getDirectMessages(for: fourWords)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "fourWords": fourWords,
                "messageCount": messages.count
            ], options: [])
        }

        // MARK: - Debug Utilities

        server.registerHandler("pollContacts") { _ in
            appState.pollForContactRequests()

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "contactCount": appState.contacts.count
            ], options: [])
        }

        server.registerHandler("clearError") { _ in
            appState.errorMessage = nil

            return try JSONSerialization.data(withJSONObject: [
                "success": true
            ], options: [])
        }

        server.registerHandler("generateIdentity") { _ in
            // Generate a random four-word identity from the dictionary
            // This uses the four-word-networking crate to generate valid dictionary words
            do {
                let fourWords = try generateIdWords()
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "fourWords": fourWords
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to generate identity: \(error.localizedDescription)")
            }
        }

        server.registerHandler("navigate") { body in
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let view = json["view"] as? String else {
                throw DebugError.invalidRequest("Missing 'view' parameter")
            }

            switch view {
            case "home":
                appState.activeView = .home
            case "contactChat":
                guard let fourWords = json["fourWords"] as? String else {
                    throw DebugError.invalidRequest("contactChat requires 'fourWords' parameter")
                }
                let displayName = json["displayName"] as? String
                appState.activeView = .contactChat(fourWords: fourWords, displayName: displayName)
            default:
                throw DebugError.invalidRequest("Unknown view: \(view). Supported: home, contactChat")
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "view": view
            ], options: [])
        }

        // MARK: - Entity Handlers

        server.registerHandler("createEntity") { body in
            // Expects: { "name": "My Org", "entityType": "organisation", "description": "optional", "parentOrgId": "optional" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let name = json["name"] as? String,
                  let entityTypeStr = json["entityType"] as? String else {
                throw DebugError.invalidRequest("Missing 'name' or 'entityType' parameter")
            }

            // Parse entity type
            let entityType: SwiftEntityType
            switch entityTypeStr.lowercased() {
            case "organisation", "organization", "org":
                entityType = .organisation
            case "group":
                entityType = .group
            case "channel":
                entityType = .channel
            case "project":
                entityType = .project
            case "person":
                entityType = .person
            default:
                throw DebugError.invalidRequest("Invalid entityType '\(entityTypeStr)'. Valid: organisation, group, channel, project, person")
            }

            let description = json["description"] as? String
            let parentOrgId = json["parentOrgId"] as? String

            appState.createEntity(name: name, type: entityType, description: description, parentOrgId: parentOrgId)

            // Reload and find the created entity
            appState.loadEntities()
            let createdEntity = appState.entities.first { $0.name == name }

            let result: [String: Any] = [
                "success": true,
                "entityId": createdEntity?.id ?? NSNull(),
                "name": name,
                "entityType": entityTypeStr,
                "entityCount": appState.entities.count
            ]
            return try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])
        }

        server.registerHandler("addEntityMember") { body in
            // Expects: { "entityId": "...", "memberFourWords": "word-word-word-word", "role": "member" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let memberFourWords = json["memberFourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'memberFourWords' parameter")
            }

            let role = (json["role"] as? String) ?? "member"

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                try client.entityAddMember(entityId: entityId, memberFourWords: memberFourWords, role: role)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": entityId,
                    "memberFourWords": memberFourWords,
                    "role": role
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to add member: \(error.localizedDescription)")
            }
        }

        server.registerHandler("removeEntityMember") { body in
            // Expects: { "entityId": "...", "memberFourWords": "word-word-word-word" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let memberFourWords = json["memberFourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'memberFourWords' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                try client.entityRemoveMember(entityId: entityId, memberFourWords: memberFourWords)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": entityId,
                    "memberFourWords": memberFourWords
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to remove member: \(error.localizedDescription)")
            }
        }

        server.registerHandler("listEntityMembers") { body in
            // Expects: { "entityId": "..." }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                let members = try client.entityListMembers(entityId: entityId)
                let memberList = members.map { member -> [String: Any] in
                    return [
                        "fourWords": member.fourWords,
                        "displayName": member.displayName ?? NSNull(),
                        "role": member.role,
                        "joinedAt": member.joinedAt
                    ]
                }
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": entityId,
                    "members": memberList,
                    "count": memberList.count
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to list members: \(error.localizedDescription)")
            }
        }

        server.registerHandler("sendEntityMessage") { body in
            // Expects: { "entityId": "...", "text": "Hello", "replyToId": "optional" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let text = json["text"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'text' parameter")
            }

            let replyToId = json["replyToId"] as? String

            appState.sendMessage(to: entityId, text: text, replyToId: replyToId)

            let result: [String: Any] = [
                "success": true,
                "entityId": entityId,
                "text": text,
                "replyToId": (replyToId as Any?) ?? NSNull()
            ]
            return try JSONSerialization.data(withJSONObject: result, options: [])
        }

        server.registerHandler("getEntityMessages") { body in
            // Expects: { "entityId": "...", "limit": 100 }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' parameter")
            }

            appState.loadMessages(for: entityId)
            let messages = appState.getMessages(for: entityId)

            let messageList = messages.map { msg -> [String: Any] in
                var messageDict: [String: Any] = [
                    "id": msg.id,
                    "text": msg.text,
                    "author": msg.author,
                    "createdAt": msg.createdAt,
                    "entityId": msg.entityId
                ]
                if let replyToId = msg.replyToId {
                    messageDict["replyToId"] = replyToId
                }
                return messageDict
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "entityId": entityId,
                "messages": messageList,
                "count": messageList.count
            ], options: [.prettyPrinted])
        }

        // MARK: - CRDT Document Handlers

        server.registerHandler("documentCreate") { body in
            // Create a new CRDT document for collaborative editing
            // Expects: { "entityId": "...", "name": "Roadmap.md", "storageMode": "shared" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let name = json["name"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'name' parameter")
            }

            let storageModeStr = (json["storageMode"] as? String) ?? "both"
            let storageMode: SwiftStorageMode
            switch storageModeStr.lowercased() {
            case "filesonly", "local", "files":
                storageMode = .filesOnly
            case "webonly", "network", "web":
                storageMode = .webOnly
            case "both", "shared":
                storageMode = .both
            default:
                storageMode = .both
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                let docId = try client.documentCreate(entityId: entityId, name: name, storageMode: storageMode)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "docId": docId,
                    "entityId": entityId,
                    "name": name,
                    "storageMode": storageModeStr
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to create document: \(error.localizedDescription)")
            }
        }

        server.registerHandler("documentList") { body in
            // List all CRDT documents for an entity
            // Expects: { "entityId": "..." } or {} for all documents
            let entityId: String?
            if let body = body,
               let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any] {
                entityId = json["entityId"] as? String
            } else {
                entityId = nil
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                let documents = try client.documentList(entityId: entityId)
                let docList = documents.map { doc -> [String: Any] in
                    return [
                        "id": doc.id,
                        "entityId": doc.entityId,
                        "name": doc.name,
                        "storageMode": "\(doc.storageMode)",
                        "createdAt": doc.createdAt,
                        "modifiedAt": doc.modifiedAt
                    ]
                }
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "documents": docList,
                    "count": docList.count
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to list documents: \(error.localizedDescription)")
            }
        }

        server.registerHandler("documentGetInfo") { body in
            // Get document metadata
            // Expects: { "docId": "..." }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let docId = json["docId"] as? String else {
                throw DebugError.invalidRequest("Missing 'docId' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                let info = try client.documentGetInfo(docId: docId)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "id": info.id,
                    "entityId": info.entityId,
                    "name": info.name,
                    "storageMode": "\(info.storageMode)",
                    "createdAt": info.createdAt,
                    "modifiedAt": info.modifiedAt
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to get document info: \(error.localizedDescription)")
            }
        }

        server.registerHandler("documentGetContent") { body in
            // Get CRDT document text content
            // Expects: { "docId": "..." }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let docId = json["docId"] as? String else {
                throw DebugError.invalidRequest("Missing 'docId' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                let content = try client.documentGetText(docId: docId)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "docId": docId,
                    "content": content,
                    "length": content.count
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to get document content: \(error.localizedDescription)")
            }
        }

        server.registerHandler("documentEdit") { body in
            // Edit CRDT document - insert or delete text
            // Expects: { "docId": "...", "operation": "insert", "position": 0, "text": "Hello" }
            //      or: { "docId": "...", "operation": "delete", "position": 0, "length": 5 }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let docId = json["docId"] as? String,
                  let operation = json["operation"] as? String,
                  let position = json["position"] as? Int else {
                throw DebugError.invalidRequest("Missing 'docId', 'operation', or 'position' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                switch operation.lowercased() {
                case "insert":
                    guard let text = json["text"] as? String else {
                        throw DebugError.invalidRequest("Insert operation requires 'text' parameter")
                    }
                    try client.documentInsertText(docId: docId, position: UInt32(position), text: text)
                    return try JSONSerialization.data(withJSONObject: [
                        "success": true,
                        "docId": docId,
                        "operation": "insert",
                        "position": position,
                        "text": text
                    ], options: [])

                case "delete":
                    guard let length = json["length"] as? Int else {
                        throw DebugError.invalidRequest("Delete operation requires 'length' parameter")
                    }
                    try client.documentDeleteText(docId: docId, position: UInt32(position), length: UInt32(length))
                    return try JSONSerialization.data(withJSONObject: [
                        "success": true,
                        "docId": docId,
                        "operation": "delete",
                        "position": position,
                        "length": length
                    ], options: [])

                default:
                    throw DebugError.invalidRequest("Invalid operation '\(operation)'. Valid: insert, delete")
                }
            } catch let error as DebugError {
                throw error
            } catch {
                throw DebugError.operationFailed("Failed to edit document: \(error.localizedDescription)")
            }
        }

        server.registerHandler("documentDelete") { body in
            // Delete a CRDT document
            // Expects: { "docId": "..." }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let docId = json["docId"] as? String else {
                throw DebugError.invalidRequest("Missing 'docId' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            do {
                try client.documentDelete(docId: docId)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "docId": docId,
                    "deleted": true
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to delete document: \(error.localizedDescription)")
            }
        }

        // MARK: - Virtual Disk Handlers (Per-Entity Storage)

        server.registerHandler("diskWriteFile") { body in
            // Write a file to an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/readme.md", "contentBase64": "SGVsbG8gV29ybGQh" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String,
                  let contentBase64 = json["contentBase64"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', 'path', or 'contentBase64' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            // Decode base64 content
            guard let data = Data(base64Encoded: contentBase64) else {
                throw DebugError.invalidRequest("Invalid base64 content")
            }

            do {
                let info = try client.diskWriteFile(entityId: entityId, diskType: diskType, path: path, data: data)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "path": info.path,
                    "name": info.name,
                    "isDirectory": info.isDirectory,
                    "sizeBytes": info.sizeBytes,
                    "modifiedAt": info.modifiedAt,
                    "contentHash": info.contentHash
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to write file: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskReadFile") { body in
            // Read a file from an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/readme.md" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', or 'path' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                let data = try client.diskReadFile(entityId: entityId, diskType: diskType, path: path)
                let contentBase64 = data.base64EncodedString()
                // Also try to decode as UTF-8 text if possible
                let contentText = String(data: Data(data), encoding: .utf8)

                var result: [String: Any] = [
                    "success": true,
                    "entityId": entityId,
                    "path": path,
                    "sizeBytes": data.count,
                    "contentBase64": contentBase64
                ]
                if let text = contentText {
                    result["contentText"] = text
                }
                return try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to read file: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskListFiles") { body in
            // List files in a directory within an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'diskType' parameter")
            }

            let path = (json["path"] as? String) ?? "/"

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                let files = try client.diskListFiles(entityId: entityId, diskType: diskType, path: path)
                let fileList = files.map { file -> [String: Any] in
                    return [
                        "path": file.path,
                        "name": file.name,
                        "isDirectory": file.isDirectory,
                        "sizeBytes": file.sizeBytes,
                        "modifiedAt": file.modifiedAt,
                        "contentHash": file.contentHash
                    ]
                }
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": entityId,
                    "diskType": diskTypeStr,
                    "path": path,
                    "files": fileList,
                    "count": fileList.count
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to list files: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskDeleteFile") { body in
            // Delete a file from an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/readme.md" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', or 'path' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                try client.diskDeleteFile(entityId: entityId, diskType: diskType, path: path)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": entityId,
                    "diskType": diskTypeStr,
                    "path": path,
                    "deleted": true
                ], options: [])
            } catch {
                throw DebugError.operationFailed("Failed to delete file: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskGetStats") { body in
            // Get storage statistics for an entity's disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'diskType' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                let stats = try client.diskGetStats(entityId: entityId, diskType: diskType)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "entityId": stats.entityId,
                    "diskType": "\(stats.diskType)",
                    "usedBytes": stats.usedBytes,
                    "fileCount": stats.fileCount,
                    "dirCount": stats.dirCount,
                    "lastModified": stats.lastModified
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to get disk stats: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskCreateDirectory") { body in
            // Create a directory in an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/subdir" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', or 'path' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                let info = try client.diskCreateDirectory(entityId: entityId, diskType: diskType, path: path)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "path": info.path,
                    "name": info.name,
                    "isDirectory": info.isDirectory,
                    "modifiedAt": info.modifiedAt
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to create directory: \(error.localizedDescription)")
            }
        }

        server.registerHandler("diskFileExists") { body in
            // Check if a file exists in an entity's virtual disk
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/readme.md" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', or 'path' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            let exists = client.diskFileExists(entityId: entityId, diskType: diskType, path: path)
            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "entityId": entityId,
                "diskType": diskTypeStr,
                "path": path,
                "exists": exists
            ], options: [])
        }

        server.registerHandler("diskGetFileInfo") { body in
            // Get file info without reading the contents
            // Expects: { "entityId": "...", "diskType": "private|public|shared", "path": "/docs/readme.md" }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let diskTypeStr = json["diskType"] as? String,
                  let path = json["path"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId', 'diskType', or 'path' parameter")
            }

            guard let client = appState.client else {
                throw DebugError.operationFailed("Client not initialized - please login first")
            }

            // Parse disk type
            let diskType: SwiftDiskType
            switch diskTypeStr.lowercased() {
            case "private":
                diskType = .private
            case "public":
                diskType = .public
            case "shared":
                diskType = .shared
            default:
                throw DebugError.invalidRequest("Invalid diskType '\(diskTypeStr)'. Valid: private, public, shared")
            }

            do {
                let info = try client.diskGetFileInfo(entityId: entityId, diskType: diskType, path: path)
                return try JSONSerialization.data(withJSONObject: [
                    "success": true,
                    "path": info.path,
                    "name": info.name,
                    "isDirectory": info.isDirectory,
                    "sizeBytes": info.sizeBytes,
                    "modifiedAt": info.modifiedAt,
                    "contentHash": info.contentHash
                ], options: [.prettyPrinted])
            } catch {
                throw DebugError.operationFailed("Failed to get file info: \(error.localizedDescription)")
            }
        }

        // MARK: - Call/WebRTC Handlers (for E2E testing)

        server.registerHandler("callState") { _ in
            // Get current call state from CallStateManager
            let callManager = CallStateManager.shared

            var result: [String: Any] = [
                "isInCall": callManager.isInCall,
                "hasIncomingCall": callManager.hasIncomingCall,
                "callQuality": callManager.callQuality.rawValue
            ]

            if let activeCall = callManager.activeCall {
                result["activeCall"] = [
                    "id": activeCall.id,
                    "peerFourWords": activeCall.peerFourWords,
                    "displayName": (activeCall.displayName ?? NSNull()) as Any,
                    "state": activeCall.state.rawValue,
                    "isVideoEnabled": activeCall.isVideoEnabled,
                    "isAudioEnabled": activeCall.isAudioEnabled,
                    "isScreenSharing": activeCall.isScreenSharing,
                    "isEntityCall": activeCall.isEntityCall,
                    "entityId": (activeCall.entityId ?? NSNull()) as Any,
                    "entityType": (activeCall.entityType ?? NSNull()) as Any
                ]
            }

            if let incomingCall = callManager.incomingCall {
                result["incomingCall"] = [
                    "id": incomingCall.id,
                    "callerFourWords": incomingCall.callerFourWords,
                    "callerDisplayName": (incomingCall.callerDisplayName ?? NSNull()) as Any,
                    "hasVideo": incomingCall.hasVideo
                ]
            }

            let participantList = callManager.participants.map { p -> [String: Any] in
                return [
                    "id": p.id,
                    "fourWords": p.fourWords,
                    "displayName": p.displayName ?? NSNull(),
                    "isVideoEnabled": p.isVideoEnabled,
                    "isAudioEnabled": p.isAudioEnabled,
                    "isSpeaking": p.isSpeaking,
                    "isScreenSharing": p.isScreenSharing
                ]
            }
            result["participants"] = participantList

            if let error = callManager.lastError {
                result["lastError"] = error
            }

            return try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted])
        }

        server.registerHandler("callInitiate") { body in
            // Initiate an outgoing call
            // Expects: { "peerFourWords": "word-word-word-word", "displayName": "optional", "hasVideo": true }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let peerFourWords = json["peerFourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'peerFourWords' parameter")
            }

            let displayName = json["displayName"] as? String
            let hasVideo = (json["hasVideo"] as? Bool) ?? false
            let callId = UUID().uuidString

            let callManager = CallStateManager.shared
            callManager.initiateCall(
                callId: callId,
                peerFourWords: peerFourWords,
                displayName: displayName,
                hasVideo: hasVideo
            )

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "callId": callId,
                "peerFourWords": peerFourWords,
                "hasVideo": hasVideo
            ], options: [])
        }

        server.registerHandler("callInitiateEntity") { body in
            // Initiate an entity-based call (group, channel, org)
            // Expects: { "entityId": "...", "entityType": "group|channel|org", "displayName": "optional", "hasVideo": true }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let entityId = json["entityId"] as? String,
                  let entityType = json["entityType"] as? String else {
                throw DebugError.invalidRequest("Missing 'entityId' or 'entityType' parameter")
            }

            let displayName = json["displayName"] as? String
            let hasVideo = (json["hasVideo"] as? Bool) ?? false
            let callId = UUID().uuidString

            let callManager = CallStateManager.shared
            callManager.initiateEntityCall(
                callId: callId,
                entityId: entityId,
                entityType: entityType,
                displayName: displayName,
                hasVideo: hasVideo
            )

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "callId": callId,
                "entityId": entityId,
                "entityType": entityType,
                "hasVideo": hasVideo
            ], options: [])
        }

        server.registerHandler("callIncoming") { body in
            // Simulate an incoming call (for testing)
            // Expects: { "callId": "optional", "callerFourWords": "word-word-word-word", "callerDisplayName": "optional", "hasVideo": true }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let callerFourWords = json["callerFourWords"] as? String else {
                throw DebugError.invalidRequest("Missing 'callerFourWords' parameter")
            }

            let callId = (json["callId"] as? String) ?? UUID().uuidString
            let callerDisplayName = json["callerDisplayName"] as? String
            let hasVideo = (json["hasVideo"] as? Bool) ?? false

            // Use WebRTCEventBridge to simulate incoming call
            let bridge = WebRTCEventBridge.shared
            bridge.onIncomingCall(
                callId: callId,
                callerFourWords: callerFourWords,
                callerDisplayName: callerDisplayName,
                hasVideo: hasVideo
            )

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "callId": callId,
                "callerFourWords": callerFourWords,
                "hasVideo": hasVideo
            ], options: [])
        }

        server.registerHandler("callAccept") { body in
            // Accept an incoming call
            // Expects: { "withVideo": true }
            let withVideo: Bool
            if let body = body,
               let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any] {
                withVideo = (json["withVideo"] as? Bool) ?? false
            } else {
                withVideo = false
            }

            let callManager = CallStateManager.shared
            guard callManager.hasIncomingCall else {
                throw DebugError.operationFailed("No incoming call to accept")
            }

            Task {
                await callManager.acceptIncomingCall(withVideo: withVideo)
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "withVideo": withVideo
            ], options: [])
        }

        server.registerHandler("callReject") { _ in
            // Reject an incoming call
            let callManager = CallStateManager.shared
            guard callManager.hasIncomingCall else {
                throw DebugError.operationFailed("No incoming call to reject")
            }

            Task {
                await callManager.rejectIncomingCall()
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "rejected": true
            ], options: [])
        }

        server.registerHandler("callEnd") { _ in
            // End the current call
            let callManager = CallStateManager.shared
            guard callManager.isInCall else {
                throw DebugError.operationFailed("No active call to end")
            }

            Task {
                await callManager.endCall()
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "ended": true
            ], options: [])
        }

        server.registerHandler("callSetVideo") { body in
            // Enable/disable video
            // Expects: { "enabled": true }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let enabled = json["enabled"] as? Bool else {
                throw DebugError.invalidRequest("Missing 'enabled' parameter")
            }

            let callManager = CallStateManager.shared
            guard callManager.isInCall else {
                throw DebugError.operationFailed("No active call")
            }

            callManager.setVideoEnabled(enabled)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "videoEnabled": enabled
            ], options: [])
        }

        server.registerHandler("callSetAudio") { body in
            // Enable/disable audio (mute/unmute)
            // Expects: { "enabled": true }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let enabled = json["enabled"] as? Bool else {
                throw DebugError.invalidRequest("Missing 'enabled' parameter")
            }

            let callManager = CallStateManager.shared
            guard callManager.isInCall else {
                throw DebugError.operationFailed("No active call")
            }

            callManager.setAudioEnabled(enabled)

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "audioEnabled": enabled
            ], options: [])
        }

        server.registerHandler("callWebRTCEvent") { body in
            // Simulate a WebRTC event (for testing call lifecycle)
            // Expects: { "eventType": "stateChanged|participantJoined|participantLeft|qualityChanged|error|ended", ... }
            guard let body = body,
                  let json = try? JSONSerialization.jsonObject(with: body) as? [String: Any],
                  let eventType = json["eventType"] as? String else {
                throw DebugError.invalidRequest("Missing 'eventType' parameter")
            }

            let bridge = WebRTCEventBridge.shared

            switch eventType.lowercased() {
            case "statechanged":
                guard let callId = json["callId"] as? String,
                      let state = json["state"] as? String else {
                    throw DebugError.invalidRequest("stateChanged requires 'callId' and 'state' parameters")
                }
                bridge.onCallStateChanged(callId: callId, state: state)

            case "participantjoined":
                guard let callId = json["callId"] as? String,
                      let participantId = json["participantId"] as? String,
                      let fourWords = json["fourWords"] as? String else {
                    throw DebugError.invalidRequest("participantJoined requires 'callId', 'participantId', and 'fourWords' parameters")
                }
                let displayName = json["displayName"] as? String
                bridge.onParticipantJoined(callId: callId, participantId: participantId, fourWords: fourWords, displayName: displayName)

            case "participantleft":
                guard let callId = json["callId"] as? String,
                      let participantId = json["participantId"] as? String else {
                    throw DebugError.invalidRequest("participantLeft requires 'callId' and 'participantId' parameters")
                }
                bridge.onParticipantLeft(callId: callId, participantId: participantId)

            case "qualitychanged":
                guard let callId = json["callId"] as? String,
                      let quality = json["quality"] as? String else {
                    throw DebugError.invalidRequest("qualityChanged requires 'callId' and 'quality' parameters")
                }
                bridge.onQualityChanged(callId: callId, quality: quality)

            case "error":
                guard let callId = json["callId"] as? String,
                      let message = json["message"] as? String else {
                    throw DebugError.invalidRequest("error requires 'callId' and 'message' parameters")
                }
                bridge.onError(callId: callId, message: message)

            case "ended":
                guard let callId = json["callId"] as? String else {
                    throw DebugError.invalidRequest("ended requires 'callId' parameter")
                }
                let reason = (json["reason"] as? String) ?? "normal"
                bridge.onCallEnded(callId: callId, reason: reason)

            case "mediachanged":
                guard let callId = json["callId"] as? String,
                      let participantId = json["participantId"] as? String else {
                    throw DebugError.invalidRequest("mediaChanged requires 'callId' and 'participantId' parameters")
                }
                let videoEnabled = (json["videoEnabled"] as? Bool) ?? false
                let audioEnabled = (json["audioEnabled"] as? Bool) ?? true
                bridge.onParticipantMediaChanged(callId: callId, participantId: participantId, videoEnabled: videoEnabled, audioEnabled: audioEnabled)

            default:
                throw DebugError.invalidRequest("Unknown eventType '\(eventType)'. Valid: stateChanged, participantJoined, participantLeft, qualityChanged, error, ended, mediaChanged")
            }

            return try JSONSerialization.data(withJSONObject: [
                "success": true,
                "eventType": eventType
            ], options: [])
        }

        print("[DebugHandlers] Registered 52 handlers")
    }
}

// MARK: - Error Types

enum DebugError: LocalizedError {
    case invalidRequest(String)
    case notFound(String)
    case operationFailed(String)

    var errorDescription: String? {
        switch self {
        case .invalidRequest(let msg): return "Invalid request: \(msg)"
        case .notFound(let msg): return "Not found: \(msg)"
        case .operationFailed(let msg): return "Operation failed: \(msg)"
        }
    }
}

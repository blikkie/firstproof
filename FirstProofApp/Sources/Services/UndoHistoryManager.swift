import Foundation
import SwiftUI
import FirstProofBridge

/// Payload holding the exact state transitions for an undoable action.
public enum UndoActionPayload: Equatable {
    case edits(before: EditSettings, after: EditSettings)
    case rating(targets: [RatingTransition])
    case status(targets: [StatusTransition])
}

public struct RatingTransition: Equatable {
    public let path: String
    public let oldRating: Int32?
    public let newRating: Int32?
    
    public init(path: String, oldRating: Int32?, newRating: Int32?) {
        self.path = path
        self.oldRating = oldRating
        self.newRating = newRating
    }
}

public struct StatusTransition: Equatable {
    public let path: String
    public let oldStatus: String
    public let newStatus: String
    
    public init(path: String, oldStatus: String, newStatus: String) {
        self.path = path
        self.oldStatus = oldStatus
        self.newStatus = newStatus
    }
}

/// A discrete, human-readable undoable history step.
public struct UndoEntry: Identifiable, Equatable {
    public let id: UUID
    public let title: String
    public let timestamp: Date
    public let primaryPhotoPath: String
    public let payload: UndoActionPayload
    
    public init(
        title: String,
        primaryPhotoPath: String,
        payload: UndoActionPayload,
        timestamp: Date = Date(),
        id: UUID = UUID()
    ) {
        self.id = id
        self.title = title
        self.timestamp = timestamp
        self.primaryPhotoPath = primaryPhotoPath
        self.payload = payload
    }
    
    public static func == (lhs: UndoEntry, rhs: UndoEntry) -> Bool {
        lhs.id == rhs.id
    }
}

/// Comprehensive, observable undo/redo and session history manager for First Proof.
@Observable
public final class UndoHistoryManager {
    public static let maxHistoryCount = 100
    
    public private(set) var undoStack: [UndoEntry] = []
    public private(set) var redoStack: [UndoEntry] = []
    
    // Transient snapshot state for coalescing slider and gesture drag scrubbing
    public var scrubStartEdits: EditSettings? = nil
    public var scrubPhotoPath: String? = nil
    
    public init() {}
    
    public var canUndo: Bool {
        !undoStack.isEmpty
    }
    
    public var canRedo: Bool {
        !redoStack.isEmpty
    }
    
    public var undoTitle: String {
        if let last = undoStack.last {
            return "Undo \(last.title)"
        }
        return "Undo"
    }
    
    public var redoTitle: String {
        if let last = redoStack.last {
            return "Redo \(last.title)"
        }
        return "Redo"
    }
    
    /// Records an atomic action to the undo stack and clears redo history.
    public func registerAction(title: String, photoPath: String, payload: UndoActionPayload) {
        let entry = UndoEntry(title: title, primaryPhotoPath: photoPath, payload: payload)
        undoStack.append(entry)
        if undoStack.count > Self.maxHistoryCount {
            undoStack.removeFirst()
        }
        redoStack.removeAll()
    }
    
    /// Commences slider or gesture scrubbing, capturing the initial edit state.
    public func beginScrub(photoPath: String, currentEdits: EditSettings) {
        if scrubStartEdits == nil {
            scrubStartEdits = currentEdits
            scrubPhotoPath = photoPath
        }
    }
    
    /// Ends slider or gesture scrubbing, committing an undo action if edits actually changed.
    @discardableResult
    public func endScrub(photoPath: String, currentEdits: EditSettings, title: String) -> UndoEntry? {
        guard let startEdits = scrubStartEdits, scrubPhotoPath == photoPath else {
            scrubStartEdits = nil
            scrubPhotoPath = nil
            return nil
        }
        
        scrubStartEdits = nil
        scrubPhotoPath = nil
        
        // Coalesce: only register if values actually changed
        if startEdits != currentEdits {
            let entry = UndoEntry(
                title: title,
                primaryPhotoPath: photoPath,
                payload: .edits(before: startEdits, after: currentEdits)
            )
            undoStack.append(entry)
            if undoStack.count > Self.maxHistoryCount {
                undoStack.removeFirst()
            }
            redoStack.removeAll()
            return entry
        }
        
        return nil
    }
    
    /// Pops the latest undo entry for execution.
    public func popUndo() -> UndoEntry? {
        guard let entry = undoStack.popLast() else { return nil }
        redoStack.append(entry)
        return entry
    }
    
    /// Pops the latest redo entry for execution.
    public func popRedo() -> UndoEntry? {
        guard let entry = redoStack.popLast() else { return nil }
        undoStack.append(entry)
        return entry
    }
    
    /// Clears the session history (e.g. when opening a new catalog directory).
    public func clearAll() {
        undoStack.removeAll()
        redoStack.removeAll()
        scrubStartEdits = nil
        scrubPhotoPath = nil
    }
}

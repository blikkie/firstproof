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

/// Dedicated, isolated in-memory darkroom edit history for a single photo.
/// Capped at 50 steps to eliminate memory bloat and prevent off-screen mutations.
@Observable
public final class PhotoEditHistory {
    public static let maxDepth = 50
    public var undoStack: [UndoEntry] = []
    public var redoStack: [UndoEntry] = []
    
    public init() {}
    
    public var canUndo: Bool { !undoStack.isEmpty }
    public var canRedo: Bool { !redoStack.isEmpty }
    
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
    
    public func pushUndo(_ entry: UndoEntry) {
        undoStack.append(entry)
        if undoStack.count > Self.maxDepth {
            undoStack.removeFirst()
        }
        redoStack.removeAll()
    }
    
    public func popUndo() -> UndoEntry? {
        guard let entry = undoStack.popLast() else { return nil }
        redoStack.append(entry)
        return entry
    }
    
    public func popRedo() -> UndoEntry? {
        guard let entry = redoStack.popLast() else { return nil }
        undoStack.append(entry)
        return entry
    }
    
    public func clear() {
        undoStack.removeAll()
        redoStack.removeAll()
    }
}

/// Comprehensive, observable undo/redo and session history manager for First Proof.
/// Supports scoped per-photo darkroom history and separate catalog session history.
@Observable
public final class UndoHistoryManager {
    public static let maxCatalogHistoryCount = 100
    public static let maxPhotoHistoryCount = 50
    
    /// Observable revision ticker to guarantee real-time UI synchronization in HistorySectionView.
    public private(set) var historyRevision: Int = 0
    
    // Per-photo isolated darkroom edit stacks
    private var photoHistories: [String: PhotoEditHistory] = [:]
    
    // Catalog-level session history for batch culling (ratings, flags)
    public private(set) var catalogUndoStack: [UndoEntry] = []
    public private(set) var catalogRedoStack: [UndoEntry] = []
    
    // Transient snapshot state for coalescing slider and gesture drag scrubbing
    public var scrubStartEdits: EditSettings? = nil
    public var scrubPhotoPath: String? = nil
    
    public init() {}
    
    // MARK: - Photo-Scoped History Access
    
    public func photoHistory(for path: String) -> PhotoEditHistory {
        if let existing = photoHistories[path] {
            return existing
        }
        let newHist = PhotoEditHistory()
        photoHistories[path] = newHist
        return newHist
    }
    
    public func undoStack(for photoPath: String?) -> [UndoEntry] {
        guard let path = photoPath, let hist = photoHistories[path] else {
            return catalogUndoStack
        }
        return hist.undoStack
    }
    
    public func redoStack(for photoPath: String?) -> [UndoEntry] {
        guard let path = photoPath, let hist = photoHistories[path] else {
            return catalogRedoStack
        }
        return hist.redoStack
    }
    
    /// Restores the exact undo and redo stacks for a photo (e.g. when cancelling an interactive crop session).
    public func restorePhotoHistory(for path: String, undoStack: [UndoEntry], redoStack: [UndoEntry]) {
        let hist = photoHistory(for: path)
        hist.undoStack = undoStack
        hist.redoStack = redoStack
        historyRevision += 1
    }
    
    // MARK: - Scoped Undo / Redo Query & Execution
    
    public func canUndo(isDetailMode: Bool, activePhotoPath: String?) -> Bool {
        if isDetailMode, let path = activePhotoPath {
            return photoHistories[path]?.canUndo ?? false
        }
        return !catalogUndoStack.isEmpty
    }
    
    public func canRedo(isDetailMode: Bool, activePhotoPath: String?) -> Bool {
        if isDetailMode, let path = activePhotoPath {
            return photoHistories[path]?.canRedo ?? false
        }
        return !catalogRedoStack.isEmpty
    }
    
    public func undoTitle(isDetailMode: Bool, activePhotoPath: String?) -> String {
        if isDetailMode, let path = activePhotoPath, let hist = photoHistories[path], let last = hist.undoStack.last {
            return "Undo \(last.title)"
        }
        if let last = catalogUndoStack.last {
            return "Undo \(last.title)"
        }
        return "Undo"
    }
    
    public func redoTitle(isDetailMode: Bool, activePhotoPath: String?) -> String {
        if isDetailMode, let path = activePhotoPath, let hist = photoHistories[path], let last = hist.redoStack.last {
            return "Redo \(last.title)"
        }
        if let last = catalogRedoStack.last {
            return "Redo \(last.title)"
        }
        return "Redo"
    }
    
    public func popUndo(isDetailMode: Bool, activePhotoPath: String?) -> UndoEntry? {
        historyRevision += 1
        if isDetailMode, let path = activePhotoPath {
            return photoHistories[path]?.popUndo()
        }
        guard let entry = catalogUndoStack.popLast() else { return nil }
        catalogRedoStack.append(entry)
        return entry
    }
    
    public func popRedo(isDetailMode: Bool, activePhotoPath: String?) -> UndoEntry? {
        historyRevision += 1
        if isDetailMode, let path = activePhotoPath {
            return photoHistories[path]?.popRedo()
        }
        guard let entry = catalogRedoStack.popLast() else { return nil }
        catalogUndoStack.append(entry)
        return entry
    }
    
    // MARK: - Action Registration
    
    /// Commences slider or gesture scrubbing, capturing the initial edit state.
    public func beginScrub(photoPath: String, currentEdits: EditSettings) {
        if scrubStartEdits == nil {
            scrubStartEdits = currentEdits
            scrubPhotoPath = photoPath
        }
    }
    
    /// Ends slider or gesture scrubbing, committing an undo action to the photo's scoped stack.
    @discardableResult
    public func endScrub(photoPath: String, currentEdits: EditSettings, title: String) -> UndoEntry? {
        guard let startEdits = scrubStartEdits, scrubPhotoPath == photoPath else {
            scrubStartEdits = nil
            scrubPhotoPath = nil
            return nil
        }
        
        scrubStartEdits = nil
        scrubPhotoPath = nil
        
        if startEdits != currentEdits {
            let entry = UndoEntry(
                title: title,
                primaryPhotoPath: photoPath,
                payload: .edits(before: startEdits, after: currentEdits)
            )
            let hist = photoHistory(for: photoPath)
            hist.pushUndo(entry)
            historyRevision += 1
            return entry
        }
        
        return nil
    }
    
    /// Records an atomic darkroom edit to the specific photo's scoped stack.
    public func registerEdit(title: String, photoPath: String, before: EditSettings, after: EditSettings) {
        guard before != after else { return }
        let entry = UndoEntry(
            title: title,
            primaryPhotoPath: photoPath,
            payload: .edits(before: before, after: after)
        )
        let hist = photoHistory(for: photoPath)
        hist.pushUndo(entry)
        historyRevision += 1
    }
    
    /// Records a catalog-level action (ratings, flags) to the catalog session stack.
    public func registerCatalogAction(title: String, photoPath: String, payload: UndoActionPayload) {
        let entry = UndoEntry(title: title, primaryPhotoPath: photoPath, payload: payload)
        catalogUndoStack.append(entry)
        if catalogUndoStack.count > Self.maxCatalogHistoryCount {
            catalogUndoStack.removeFirst()
        }
        catalogRedoStack.removeAll()
        historyRevision += 1
    }
    
    /// Universal action registration dispatcher routing edits to photo stack and ratings/status to catalog stack.
    public func registerAction(title: String, photoPath: String, payload: UndoActionPayload) {
        switch payload {
        case .edits(let before, let after):
            registerEdit(title: title, photoPath: photoPath, before: before, after: after)
        case .rating, .status:
            registerCatalogAction(title: title, photoPath: photoPath, payload: payload)
        }
    }
    
    /// Clears the entire session history (e.g. when opening a new catalog directory).
    public func clearAll() {
        photoHistories.removeAll()
        catalogUndoStack.removeAll()
        catalogRedoStack.removeAll()
        scrubStartEdits = nil
        scrubPhotoPath = nil
        historyRevision += 1
    }
    
    /// Clears history for a specific photo.
    public func clear(for photoPath: String) {
        photoHistories[photoPath]?.clear()
        historyRevision += 1
    }
}

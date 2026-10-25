// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("Undo & Redo History Invariants")
struct UndoHistoryTests {
    
    @Test("Single edit registration pushes to undo stack and clears redo stack")
    func testSingleEditRegistration() {
        let manager = UndoHistoryManager()
        let path = "/tmp/photo1.cr2"
        let initialEdits = EditSettings.defaultIdentity
        var editedEdits = initialEdits
        editedEdits.exposure = 1.0
        
        manager.registerEdit(
            title: "Exposure +1.0 EV",
            photoPath: path,
            before: initialEdits,
            after: editedEdits
        )
        
        #expect(manager.canUndo(isDetailMode: true, activePhotoPath: path))
        #expect(!manager.canRedo(isDetailMode: true, activePhotoPath: path))
        #expect(manager.undoTitle(isDetailMode: true, activePhotoPath: path) == "Undo Exposure +1.0 EV")
        
        let stack = manager.undoStack(for: path)
        #expect(stack.count == 1)
        #expect(stack[0].title == "Exposure +1.0 EV")
    }
    
    @Test("Slider scrub lifecycle coalesces continuous drag frames into a single atomic undo state")
    func testSliderScrubbingCoalescing() {
        let manager = UndoHistoryManager()
        let path = "/tmp/photo1.cr2"
        let startEdits = EditSettings.defaultIdentity
        
        // 1. User begins dragging a slider (e.g. Exposure)
        manager.beginScrub(photoPath: path, currentEdits: startEdits)
        #expect(manager.scrubStartEdits != nil)
        #expect(manager.scrubPhotoPath == path)
        
        // 2. Continuous drag events fire 50 times during mouse move
        var intermediateEdits = startEdits
        for ev in 1...50 {
            intermediateEdits.exposure = Float(ev) * 0.05
        }
        
        // 3. User releases mouse pointer to end drag scrub
        manager.endScrub(photoPath: path, currentEdits: intermediateEdits, title: "Adjust Exposure")
        
        // 4. Invariant: Scrub state cleared
        #expect(manager.scrubStartEdits == nil)
        #expect(manager.scrubPhotoPath == nil)
        
        // 5. Invariant: Exactly ONE undo entry exists, with the initial state and final state
        let stack = manager.undoStack(for: path)
        #expect(stack.count == 1)
        #expect(stack[0].title == "Adjust Exposure")
        
        if case let .edits(before, after) = stack[0].payload {
            #expect(before.exposure == 0.0)
            #expect(after.exposure == intermediateEdits.exposure)
        } else {
            Issue.record("Expected .edits payload in undo entry")
        }
    }
    
    @Test("Cancelling a slider scrub discards transient drag frames without touching history")
    func testSliderScrubCancellation() {
        let manager = UndoHistoryManager()
        let path = "/tmp/photo1.cr2"
        let startEdits = EditSettings.defaultIdentity
        
        manager.beginScrub(photoPath: path, currentEdits: startEdits)
        #expect(manager.scrubStartEdits != nil)
        
        manager.cancelScrub()
        
        #expect(manager.scrubStartEdits == nil)
        #expect(manager.scrubPhotoPath == nil)
        #expect(manager.undoStack(for: path).isEmpty)
    }
    
    @Test("Undo and redo cycle correctly moves entries between stacks")
    func testUndoRedoLifecycle() {
        let manager = UndoHistoryManager()
        let path = "/tmp/photo1.cr2"
        let e0 = EditSettings.defaultIdentity
        var e1 = e0
        e1.contrast = 20.0
        
        manager.registerEdit(title: "Contrast +20", photoPath: path, before: e0, after: e1)
        #expect(manager.canUndo(isDetailMode: true, activePhotoPath: path))
        #expect(!manager.canRedo(isDetailMode: true, activePhotoPath: path))
        
        // Execute Undo
        guard let undoEntry = manager.popUndo(isDetailMode: true, activePhotoPath: path) else {
            Issue.record("Undo entry should be available")
            return
        }
        #expect(undoEntry.title == "Contrast +20")
        #expect(!manager.canUndo(isDetailMode: true, activePhotoPath: path))
        #expect(manager.canRedo(isDetailMode: true, activePhotoPath: path))
        #expect(manager.redoTitle(isDetailMode: true, activePhotoPath: path) == "Redo Contrast +20")
        
        // Execute Redo
        guard let redoEntry = manager.popRedo(isDetailMode: true, activePhotoPath: path) else {
            Issue.record("Redo entry should be available")
            return
        }
        #expect(redoEntry.title == "Contrast +20")
        #expect(manager.canUndo(isDetailMode: true, activePhotoPath: path))
        #expect(!manager.canRedo(isDetailMode: true, activePhotoPath: path))
    }
    
    @Test("Photo history stack caps at maximum limit (50 entries)")
    func testMaxPhotoHistoryLimit() {
        let manager = UndoHistoryManager()
        let path = "/tmp/photo1.cr2"
        var current = EditSettings.defaultIdentity
        
        // Push 60 distinct edits
        for i in 1...60 {
            var next = current
            next.exposure = Float(i) * 0.1
            manager.registerEdit(title: "Step \(i)", photoPath: path, before: current, after: next)
            current = next
        }
        
        let stack = manager.undoStack(for: path)
        #expect(stack.count == UndoHistoryManager.maxPhotoHistoryCount)
        #expect(stack.last?.title == "Step 60")
        #expect(stack.first?.title == "Step 11") // Earliest 10 dropped FIFO
    }
    
    @Test("Per-photo edit histories remain strictly isolated")
    func testPerPhotoStackIsolation() {
        let manager = UndoHistoryManager()
        let pathA = "/tmp/photo_a.cr2"
        let pathB = "/tmp/photo_b.cr2"
        let e0 = EditSettings.defaultIdentity
        var eA = e0; eA.exposure = 0.5
        var eB = e0; eB.contrast = 15.0
        
        manager.registerEdit(title: "Exposure A", photoPath: pathA, before: e0, after: eA)
        manager.registerEdit(title: "Contrast B", photoPath: pathB, before: e0, after: eB)
        
        let stackA = manager.undoStack(for: pathA)
        let stackB = manager.undoStack(for: pathB)
        
        #expect(stackA.count == 1)
        #expect(stackA[0].title == "Exposure A")
        
        #expect(stackB.count == 1)
        #expect(stackB[0].title == "Contrast B")
    }
}

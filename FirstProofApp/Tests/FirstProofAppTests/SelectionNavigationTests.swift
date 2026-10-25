// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import Foundation
import FirstProofBridge
@testable import FirstProofApp

@Suite("Selection & Boundary Navigation Invariants")
@MainActor
struct SelectionNavigationTests {
    
    @Test("Navigating empty catalog does not crash or select non-existent photos")
    func testEmptyCatalogNavigation() {
        let vm = AppViewModel()
        vm.photos = []
        vm.selectedPhoto = nil
        vm.selectedPhotoPaths = []
        
        vm.selectNextPhoto()
        #expect(vm.selectedPhoto == nil)
        #expect(vm.selectedPhotoPaths.isEmpty)
        
        vm.selectPreviousPhoto()
        #expect(vm.selectedPhoto == nil)
        #expect(vm.selectedPhotoPaths.isEmpty)
    }
    
    @Test("Navigating single photo catalog clamps strictly to that photo without reloading")
    func testSinglePhotoNavigation() {
        let vm = AppViewModel()
        let photo = PhotoRecord.mock(id: 1, path: "/tmp/sample_001.cr2")
        vm.photos = [photo]
        vm.selectPhoto(photo)
        
        #expect(vm.selectedPhoto?.filePath == photo.filePath)
        
        // Next photo clamps to index 0
        vm.selectNextPhoto()
        #expect(vm.selectedPhoto?.filePath == photo.filePath)
        
        // Previous photo clamps to index 0
        vm.selectPreviousPhoto()
        #expect(vm.selectedPhoto?.filePath == photo.filePath)
    }
    
    @Test("Forward and backward navigation step cleanly and clamp at catalog boundaries")
    func testBoundaryClampingNavigation() {
        let vm = AppViewModel()
        let p0 = PhotoRecord.mock(id: 1, path: "/tmp/p0.cr2")
        let p1 = PhotoRecord.mock(id: 2, path: "/tmp/p1.cr2")
        let p2 = PhotoRecord.mock(id: 3, path: "/tmp/p2.cr2")
        vm.photos = [p0, p1, p2]
        vm.selectPhoto(p0)
        
        #expect(vm.selectedPhoto?.filePath == p0.filePath)
        
        // Boundary Guard Left: Arrowing backward at index 0 remains at index 0
        vm.selectPreviousPhoto()
        #expect(vm.selectedPhoto?.filePath == p0.filePath)
        
        // Step forward to index 1
        vm.selectNextPhoto()
        #expect(vm.selectedPhoto?.filePath == p1.filePath)
        
        // Step forward to index 2 (last)
        vm.selectNextPhoto()
        #expect(vm.selectedPhoto?.filePath == p2.filePath)
        
        // Boundary Guard Right: Arrowing forward at index 2 remains at index 2
        vm.selectNextPhoto()
        #expect(vm.selectedPhoto?.filePath == p2.filePath)
        
        // Step backward to index 1
        vm.selectPreviousPhoto()
        #expect(vm.selectedPhoto?.filePath == p1.filePath)
    }
    
    @Test("Range selection selects all contiguous photos between anchor and target")
    func testRangeSelection() {
        let vm = AppViewModel()
        let p0 = PhotoRecord.mock(id: 1, path: "/tmp/p0.cr2")
        let p1 = PhotoRecord.mock(id: 2, path: "/tmp/p1.cr2")
        let p2 = PhotoRecord.mock(id: 3, path: "/tmp/p2.cr2")
        let p3 = PhotoRecord.mock(id: 4, path: "/tmp/p3.cr2")
        vm.photos = [p0, p1, p2, p3]
        
        vm.selectPhoto(p1) // Anchor at index 1
        vm.selectPhotoRange(to: p3) // Range to index 3
        
        #expect(vm.selectedPhotoPaths.count == 3)
        #expect(vm.selectedPhotoPaths.contains(p1.filePath))
        #expect(vm.selectedPhotoPaths.contains(p2.filePath))
        #expect(vm.selectedPhotoPaths.contains(p3.filePath))
        #expect(!vm.selectedPhotoPaths.contains(p0.filePath))
        #expect(vm.selectedPhoto?.filePath == p3.filePath)
    }
}

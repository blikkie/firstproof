// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
import AppKit
@testable import FirstProofApp

@Suite("Keyboard First-Responder Shielding Invariants")
struct KeyboardShieldingTests {
    
    @Test("Nil first responder allows single-key canvas shortcuts")
    func testNilResponderDoesNotShield() {
        #expect(!KeyboardShielding.shouldShieldKeyboardEvent(responder: nil))
    }
    
    @Test("Generic NSView canvas responder allows single-key canvas shortcuts")
    func testGenericViewDoesNotShield() {
        let view = NSView()
        #expect(!KeyboardShielding.shouldShieldKeyboardEvent(responder: view))
    }
    
    @Test("Active NSTextView shields single-key culling shortcuts")
    func testTextViewShields() {
        let textView = NSTextView()
        #expect(KeyboardShielding.shouldShieldKeyboardEvent(responder: textView))
    }
    
    @Test("Active NSTextField shields single-key culling shortcuts")
    func testTextFieldShields() {
        let textField = NSTextField()
        #expect(KeyboardShielding.shouldShieldKeyboardEvent(responder: textField))
    }
    
    @Test("Active NSSearchField shields single-key culling shortcuts")
    func testSearchFieldShields() {
        let searchField = NSSearchField()
        #expect(KeyboardShielding.shouldShieldKeyboardEvent(responder: searchField))
    }
    
    private class MockTextInputView: NSView, NSTextInputClient {
        func insertText(_ string: Any, replacementRange: NSRange) {}
        func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {}
        func unmarkText() {}
        func selectedRange() -> NSRange { NSRange(location: 0, length: 0) }
        func markedRange() -> NSRange { NSRange(location: NSNotFound, length: 0) }
        func hasMarkedText() -> Bool { false }
        func attributedSubstring(forProposedRange range: NSRange, actualRange: NSRangePointer?) -> NSAttributedString? { nil }
        func validAttributesForMarkedText() -> [NSAttributedString.Key] { [] }
        func firstRect(forCharacterRange range: NSRange, actualRange: NSRangePointer?) -> NSRect { .zero }
        func characterIndex(for point: NSPoint) -> Int { 0 }
    }
    
    @Test("Any NSTextInputClient implementation shields single-key culling shortcuts")
    func testCustomTextInputClientShields() {
        let customInput = MockTextInputView()
        #expect(KeyboardShielding.shouldShieldKeyboardEvent(responder: customInput))
    }
}

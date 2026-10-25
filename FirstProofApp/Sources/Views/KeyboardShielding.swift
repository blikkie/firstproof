// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import AppKit

/// Desktop keyboard event first-responder shielding utilities.
/// Enforces that single-key culling shortcuts (1..5, P/X/U, C, Z, [, ], \)
/// never fire while the photographer is typing into editable text fields.
public enum KeyboardShielding {
    /// Determines whether the active first responder is an editable text input
    /// (e.g. metadata fields, caption editor, search field).
    public static func shouldShieldKeyboardEvent(responder: NSResponder?) -> Bool {
        guard let responder = responder else { return false }
        return responder is NSTextInputClient || responder is NSTextView || responder is NSTextField || responder is NSSearchField
    }
}

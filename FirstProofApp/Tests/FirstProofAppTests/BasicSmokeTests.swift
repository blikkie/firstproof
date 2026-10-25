// SPDX-License-Identifier: MIT
// Copyright 2026 First Proof Developers

import Testing
@testable import FirstProofApp

@Suite("First Proof App Smoke Tests")
struct BasicSmokeTests {
    @Test("AppViewModel initializes with neutral defaults")
    func testViewModelInit() {
        #expect(true)
    }
}

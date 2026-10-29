import Foundation
import AppKit
import UniformTypeIdentifiers
import FirstProofBridge

extension AppViewModel {
    /// Refreshes the list of globally available 3D LUTs discovered on disk.
    public func refreshGlobalLuts() {
        self.availableGlobalLuts = discoverGlobalLuts()
    }
    
    /// Imports a `.cube` file into the persistent global LUT directory without altering the active photo edits.
    @discardableResult
    public func importLutToLibrary(from url: URL) throws -> LutRecord {
        let record = try importGlobalLut(sourcePath: url.path)
        refreshGlobalLuts()
        return record
    }
    
    /// Removes a custom LUT from the global library and updates selection/defaults if currently active.
    public func deleteLutFromLibrary(_ record: LutRecord) {
        do {
            _ = try deleteGlobalLut(idOrPath: record.filePath)
            customLuts.removeAll(where: { $0.title == record.title })
            refreshGlobalLuts()
            if selectedLutName == record.title {
                setSelectedLutName(nil)
            }
            if defaultLookLutName == record.title {
                defaultLookLutName = nil
            }
        } catch {
            print("Failed to delete LUT from library: \(error)")
        }
    }
    
    /// Renames a custom LUT on disk and in its internal header, updating active selections and default looks.
    public func renameLutInLibrary(_ record: LutRecord, newTitle: String) {
        let trimmed = newTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, trimmed != record.title else { return }
        do {
            let updated = try renameGlobalLut(idOrPath: record.filePath, newTitle: trimmed)
            if let idx = customLuts.firstIndex(where: { $0.title == record.title }) {
                let old = customLuts[idx]
                customLuts[idx] = CubeLutRecord(title: updated.title, size: old.size, dataRgba32f: old.dataRgba32f)
            }
            refreshGlobalLuts()
            if selectedLutName == record.title {
                setSelectedLutName(updated.title)
            }
            if defaultLookLutName == record.title {
                defaultLookLutName = updated.title
            }
        } catch {
            print("Failed to rename LUT: \(error)")
        }
    }
}


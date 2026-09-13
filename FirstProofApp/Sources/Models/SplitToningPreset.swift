import Foundation

public struct SplitToningPreset: Identifiable, Hashable {
    public let id: String
    public let name: String
    public let subtitle: String
    public let shadowHue: Float
    public let shadowSat: Float
    public let highlightHue: Float
    public let highlightSat: Float
    public let balance: Float

    public init(
        id: String,
        name: String,
        subtitle: String,
        shadowHue: Float,
        shadowSat: Float,
        highlightHue: Float,
        highlightSat: Float,
        balance: Float
    ) {
        self.id = id
        self.name = name
        self.subtitle = subtitle
        self.shadowHue = shadowHue
        self.shadowSat = shadowSat
        self.highlightHue = highlightHue
        self.highlightSat = highlightSat
        self.balance = balance
    }

    public static let builtInPresets: [SplitToningPreset] = [
        SplitToningPreset(
            id: "selenium_silver",
            name: "Selenium & Warm Silver",
            subtitle: "Cool plum-black shadow D-max with warm paper base highlights",
            shadowHue: 235.0,
            shadowSat: 28.0,
            highlightHue: 45.0,
            highlightSat: 16.0,
            balance: 10.0
        ),
        SplitToningPreset(
            id: "sulfide_sepia",
            name: "Sulfide Sepia",
            subtitle: "Traditional chemical sulfur toner with warm umber shadows",
            shadowHue: 35.0,
            shadowSat: 32.0,
            highlightHue: 48.0,
            highlightSat: 18.0,
            balance: -15.0
        ),
        SplitToningPreset(
            id: "cyanotype_gold",
            name: "Cyanotype & Gold",
            subtitle: "Historic Prussian blue shadows with pale protective gold highlights",
            shadowHue: 210.0,
            shadowSat: 36.0,
            highlightHue: 50.0,
            highlightSat: 14.0,
            balance: 20.0
        ),
        SplitToningPreset(
            id: "platinum_palladium",
            name: "Platinum / Palladium",
            subtitle: "Subtle cool shadow depth with luminous platinum highlight warmth",
            shadowHue: 220.0,
            shadowSat: 16.0,
            highlightHue: 40.0,
            highlightSat: 22.0,
            balance: 5.0
        ),
        SplitToningPreset(
            id: "photographic_print",
            name: "Photographic Print Stock",
            subtitle: "Classic photochemical print stock with teal-cyan shadow bias and amber sun",
            shadowHue: 195.0,
            shadowSat: 22.0,
            highlightHue: 42.0,
            highlightSat: 16.0,
            balance: 0.0
        )
    ]
}

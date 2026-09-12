import Foundation

public struct FilmGrainPreset: Identifiable, Hashable {
    public let id: String
    public let name: String
    public let subtitle: String
    public let amount: Float
    public let size: Float
    public let roughness: Float
    public let chroma: Float

    public init(
        id: String,
        name: String,
        subtitle: String,
        amount: Float,
        size: Float,
        roughness: Float,
        chroma: Float
    ) {
        self.id = id
        self.name = name
        self.subtitle = subtitle
        self.amount = amount
        self.size = size
        self.roughness = roughness
        self.chroma = chroma
    }

    public static let builtInPresets: [FilmGrainPreset] = [
        FilmGrainPreset(
            id: "p400_portrait",
            name: "P-400 Portrait",
            subtitle: "Fine 35mm color negative grain with gentle midtone clumping",
            amount: 22.0,
            size: 0.8,
            roughness: 35.0,
            chroma: 15.0
        ),
        FilmGrainPreset(
            id: "tx400_street",
            name: "TX-400 Street Classic",
            subtitle: "High-contrast monochrome silver grain with classic acutance",
            amount: 32.0,
            size: 1.0,
            roughness: 45.0,
            chroma: 0.0
        ),
        FilmGrainPreset(
            id: "cinema800_tungsten",
            name: "Cinema 800 Tungsten",
            subtitle: "Coarse cinematic film grain with subtle chromatic dye clouds",
            amount: 28.0,
            size: 1.3,
            roughness: 40.0,
            chroma: 25.0
        ),
        FilmGrainPreset(
            id: "chrome50_fine",
            name: "Chrome 50 Ultra-Fine",
            subtitle: "Microscopic slide film grain; razor-sharp and subtle",
            amount: 14.0,
            size: 0.6,
            roughness: 20.0,
            chroma: 5.0
        ),
        FilmGrainPreset(
            id: "silver400_documentary",
            name: "Silver 400 Documentary",
            subtitle: "Rich, tactile analog silver gelatin texture",
            amount: 26.0,
            size: 1.0,
            roughness: 40.0,
            chroma: 0.0
        ),
        FilmGrainPreset(
            id: "vintage16mm_gritty",
            name: "16mm Gritty Vintage",
            subtitle: "Heavy vintage film noise for stylized raw textures",
            amount: 46.0,
            size: 1.6,
            roughness: 60.0,
            chroma: 20.0
        )
    ]
}

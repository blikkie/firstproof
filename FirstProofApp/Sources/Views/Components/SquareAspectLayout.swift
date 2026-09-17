import SwiftUI

/// A native SwiftUI custom Layout that forces its subviews into a strict 1:1 square aspect ratio.
///
/// In macOS inspector sidebars (`ScrollView`/`VStack`), `GeometryReader` is greedy and unconstrained along
/// the vertical axis. `SquareAspectLayout` participates natively in SwiftUI's layout negotiation:
/// its `sizeThatFits` computes a deterministic square dimension `min(proposedWidth, proposedHeight ?? proposedWidth)`,
/// reserving exact vertical layout space in parent scroll views without relying on `Color.clear` overlay hacks.
public struct SquareAspectLayout: Layout {
    public init() {}
    
    public func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? 200.0
        let height = proposal.height ?? width
        let dimension = max(1.0, min(width, height))
        return CGSize(width: dimension, height: dimension)
    }
    
    public func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let dimension = max(1.0, min(bounds.width, bounds.height))
        let squareOrigin = CGPoint(
            x: bounds.midX - dimension / 2.0,
            y: bounds.midY - dimension / 2.0
        )
        let exactProposal = ProposedViewSize(width: dimension, height: dimension)
        
        for subview in subviews {
            subview.place(at: squareOrigin, proposal: exactProposal)
        }
    }
}

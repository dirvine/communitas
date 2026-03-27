import SwiftUI

/// Renders a subset of Markdown commonly used in chat messages:
///   - **bold**, *italic*, `inline code`
///   - @mention badges
///   - > blockquotes (line-level)
///   - ```code blocks```
struct MarkdownMessageView: View {
    let text: String

    var body: some View {
        // Split out fenced code blocks first, then render each segment.
        let segments = parseTopLevel(text)

        VStack(alignment: .leading, spacing: 4) {
            ForEach(Array(segments.enumerated()), id: \.offset) { _, segment in
                switch segment {
                case .codeBlock(let code, let lang):
                    CodeBlockView(code: code, language: lang)
                case .blockquote(let content):
                    BlockquoteView(content: content)
                case .inline(let raw):
                    inlineText(raw)
                }
            }
        }
    }

    // MARK: - Inline text rendering

    @ViewBuilder
    private func inlineText(_ raw: String) -> some View {
        // Build a SwiftUI Text by parsing tokens
        let tokens = tokenizeInline(raw)
        tokens.reduce(Text("")) { result, token in
            result + tokenToText(token)
        }
        .font(.body)
        .textSelection(.enabled)
        .fixedSize(horizontal: false, vertical: true)
    }

    private func tokenToText(_ token: InlineToken) -> Text {
        switch token {
        case .plain(let s):
            return Text(s)
        case .bold(let s):
            return Text(s).bold()
        case .italic(let s):
            return Text(s).italic()
        case .code(let s):
            return Text(s)
                .font(.system(.body, design: .monospaced))
                .foregroundColor(.primary)
        case .mention(let name):
            // Mentions rendered as accented text inline; pill badges need AttributedString
            return Text("@\(name)")
                .foregroundColor(.accentColor)
                .fontWeight(.semibold)
        }
    }

    // MARK: - Tokenizer types

    private enum TopLevelSegment {
        case codeBlock(code: String, language: String?)
        case blockquote(content: String)
        case inline(String)
    }

    private enum InlineToken {
        case plain(String)
        case bold(String)
        case italic(String)
        case code(String)
        case mention(String)
    }

    // MARK: - Top-level parser (code blocks + blockquotes)

    private func parseTopLevel(_ input: String) -> [TopLevelSegment] {
        var segments: [TopLevelSegment] = []
        var remaining = input[input.startIndex...]

        while !remaining.isEmpty {
            // Check for fenced code block ```
            if let codeRange = remaining.range(of: "```") {
                // Everything before the fence is inline or blockquote
                let before = String(remaining[remaining.startIndex..<codeRange.lowerBound])
                if !before.isEmpty {
                    segments.append(contentsOf: parseBlockquotes(before))
                }
                // Find closing ```
                let afterFence = remaining[codeRange.upperBound...]
                // First line may be the language identifier
                let firstNewline = afterFence.firstIndex(of: "\n")
                let lang: String?
                let codeStart: String.Index
                if let nl = firstNewline {
                    let langCandidate = String(afterFence[afterFence.startIndex..<nl]).trimmingCharacters(in: .whitespaces)
                    lang = langCandidate.isEmpty ? nil : langCandidate
                    codeStart = afterFence.index(after: nl)
                } else {
                    lang = nil
                    codeStart = afterFence.startIndex
                }
                let codeBody = afterFence[codeStart...]
                if let closingRange = codeBody.range(of: "```") {
                    let code = String(codeBody[codeBody.startIndex..<closingRange.lowerBound])
                    segments.append(.codeBlock(code: code, language: lang))
                    remaining = codeBody[closingRange.upperBound...]
                } else {
                    // No closing fence — treat the rest as a code block
                    segments.append(.codeBlock(code: String(codeBody), language: lang))
                    remaining = remaining[remaining.endIndex...]
                }
            } else {
                segments.append(contentsOf: parseBlockquotes(String(remaining)))
                remaining = remaining[remaining.endIndex...]
            }
        }
        return segments
    }

    private func parseBlockquotes(_ input: String) -> [TopLevelSegment] {
        var segments: [TopLevelSegment] = []
        var inlineBuffer = ""

        for line in input.components(separatedBy: "\n") {
            if line.hasPrefix("> ") || line == ">" {
                // Flush inline buffer
                if !inlineBuffer.isEmpty {
                    segments.append(.inline(inlineBuffer.trimmingCharacters(in: .newlines)))
                    inlineBuffer = ""
                }
                let content = line.hasPrefix("> ") ? String(line.dropFirst(2)) : ""
                segments.append(.blockquote(content: content))
            } else {
                inlineBuffer += (inlineBuffer.isEmpty ? "" : "\n") + line
            }
        }
        if !inlineBuffer.isEmpty {
            segments.append(.inline(inlineBuffer))
        }
        return segments
    }

    // MARK: - Inline tokenizer

    private func tokenizeInline(_ input: String) -> [InlineToken] {
        var tokens: [InlineToken] = []
        var remaining = input[input.startIndex...]

        while !remaining.isEmpty {
            // Try each pattern in priority order
            if let (token, rest) = matchBold(remaining) {
                tokens.append(token)
                remaining = rest
            } else if let (token, rest) = matchItalic(remaining) {
                tokens.append(token)
                remaining = rest
            } else if let (token, rest) = matchCode(remaining) {
                tokens.append(token)
                remaining = rest
            } else if let (token, rest) = matchMention(remaining) {
                tokens.append(token)
                remaining = rest
            } else {
                // Consume one character as plain text
                let char = remaining.removeFirst()
                if case .plain(let s) = tokens.last {
                    tokens[tokens.count - 1] = .plain(s + String(char))
                } else {
                    tokens.append(.plain(String(char)))
                }
            }
        }
        return tokens
    }

    private func matchBold(_ s: Substring) -> (InlineToken, Substring)? {
        guard s.hasPrefix("**") else { return nil }
        let body = s.dropFirst(2)
        guard let end = body.range(of: "**") else { return nil }
        let content = String(body[body.startIndex..<end.lowerBound])
        guard !content.isEmpty else { return nil }
        return (.bold(content), body[end.upperBound...])
    }

    private func matchItalic(_ s: Substring) -> (InlineToken, Substring)? {
        guard s.hasPrefix("*") && !s.hasPrefix("**") else { return nil }
        let body = s.dropFirst(1)
        // Find closing * that is not **
        var idx = body.startIndex
        while idx < body.endIndex {
            if body[idx] == "*" {
                let content = String(body[body.startIndex..<idx])
                guard !content.isEmpty else { return nil }
                let next = body.index(after: idx)
                // Ensure it's not **
                if next < body.endIndex && body[next] == "*" { return nil }
                return (.italic(content), body[next...])
            }
            idx = body.index(after: idx)
        }
        return nil
    }

    private func matchCode(_ s: Substring) -> (InlineToken, Substring)? {
        guard s.hasPrefix("`") && !s.hasPrefix("```") else { return nil }
        let body = s.dropFirst(1)
        guard let end = body.range(of: "`") else { return nil }
        let content = String(body[body.startIndex..<end.lowerBound])
        guard !content.isEmpty else { return nil }
        return (.code(content), body[end.upperBound...])
    }

    private func matchMention(_ s: Substring) -> (InlineToken, Substring)? {
        guard s.hasPrefix("@") else { return nil }
        let body = s.dropFirst(1)
        // Mention name: letters, digits, underscores, hyphens, dots
        var idx = body.startIndex
        while idx < body.endIndex {
            let c = body[idx]
            guard c.isLetter || c.isNumber || c == "_" || c == "-" || c == "." else { break }
            idx = body.index(after: idx)
        }
        let name = String(body[body.startIndex..<idx])
        guard !name.isEmpty else { return nil }
        return (.mention(name), body[idx...])
    }
}

// MARK: - Code Block View

private struct CodeBlockView: View {
    let code: String
    let language: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if let language {
                Text(language)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 10)
                    .padding(.top, 6)
                    .padding(.bottom, 2)
            }
            ScrollView(.horizontal, showsIndicators: false) {
                Text(code.trimmingCharacters(in: .newlines))
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.primary)
                    .textSelection(.enabled)
                    .padding(.horizontal, 10)
                    .padding(.vertical, language == nil ? 8 : 4)
                    .padding(.bottom, language != nil ? 8 : 0)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .background(Color.secondary.opacity(0.12), in: RoundedRectangle(cornerRadius: 6))
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .strokeBorder(Color.secondary.opacity(0.15), lineWidth: 1)
        )
    }
}

// MARK: - Blockquote View

private struct BlockquoteView: View {
    let content: String

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            RoundedRectangle(cornerRadius: 2)
                .fill(Color.accentColor.opacity(0.5))
                .frame(width: 3)
            Text(content)
                .font(.body)
                .foregroundStyle(.secondary)
                .italic()
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 2)
    }
}

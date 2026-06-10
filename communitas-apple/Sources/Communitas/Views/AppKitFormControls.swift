import AppKit
import SwiftUI

struct AppKitInlineButton: NSViewRepresentable {
    let title: String
    let systemSymbolName: String?
    let accessibilityIdentifier: String
    let onPress: () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(onPress: onPress)
    }

    func makeNSView(context: Context) -> NSButton {
        let button = NSButton(title: title, target: context.coordinator, action: #selector(Coordinator.press(_:)))
        button.bezelStyle = .rounded
        button.controlSize = .small
        if let systemSymbolName {
            button.image = NSImage(systemSymbolName: systemSymbolName, accessibilityDescription: title)
            button.imagePosition = .imageLeading
        }
        button.setAccessibilityIdentifier(accessibilityIdentifier)
        return button
    }

    func updateNSView(_ button: NSButton, context: Context) {
        button.title = title
        context.coordinator.onPress = onPress
    }

    final class Coordinator: NSObject {
        var onPress: () -> Void

        init(onPress: @escaping () -> Void) {
            self.onPress = onPress
        }

        @objc func press(_ sender: NSButton) {
            onPress()
        }
    }
}

struct AppKitPageCreatePanel: NSViewRepresentable {
    let placeholder: String
    let buttonTitle: String
    let accessibilityPrefix: String
    let onSubmit: (String) -> Bool

    func makeCoordinator() -> Coordinator {
        Coordinator(onSubmit: onSubmit)
    }

    func makeNSView(context: Context) -> NSStackView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .width
        stack.spacing = 6
        stack.translatesAutoresizingMaskIntoConstraints = false

        let field = NSTextField()
        field.placeholderString = placeholder
        field.bezelStyle = .roundedBezel
        field.delegate = context.coordinator
        field.setAccessibilityIdentifier("\(accessibilityPrefix)-field")

        let button = NSButton(
            title: buttonTitle,
            target: context.coordinator,
            action: #selector(Coordinator.submit(_:))
        )
        button.bezelStyle = .rounded
        button.controlSize = .small
        button.image = NSImage(systemSymbolName: "plus", accessibilityDescription: buttonTitle)
        button.imagePosition = .imageLeading
        button.setAccessibilityIdentifier("\(accessibilityPrefix)-button")

        let spacer = NSView()
        spacer.setContentHuggingPriority(.defaultLow, for: .horizontal)
        let buttonRow = NSStackView(views: [spacer, button])
        buttonRow.orientation = .horizontal
        buttonRow.alignment = .centerY
        buttonRow.distribution = .fill

        stack.addArrangedSubview(field)
        stack.addArrangedSubview(buttonRow)

        context.coordinator.field = field
        context.coordinator.button = button
        context.coordinator.refreshButtonState()

        return stack
    }

    func updateNSView(_ stack: NSStackView, context: Context) {
        context.coordinator.onSubmit = onSubmit
        context.coordinator.refreshButtonState()
    }

    final class Coordinator: NSObject, NSTextFieldDelegate {
        weak var field: NSTextField?
        weak var button: NSButton?
        var onSubmit: (String) -> Bool

        init(onSubmit: @escaping (String) -> Bool) {
            self.onSubmit = onSubmit
        }

        func controlTextDidChange(_ notification: Notification) {
            refreshButtonState()
        }

        func controlTextDidEndEditing(_ notification: Notification) {
            refreshButtonState()
        }

        func refreshButtonState() {
            let hasText = !(field?.stringValue.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ?? true)
            button?.isEnabled = hasText
        }

        @objc func submit(_ sender: NSButton) {
            guard let field else { return }
            if onSubmit(field.stringValue) {
                field.stringValue = ""
                refreshButtonState()
            }
        }
    }
}

struct AppKitTextEditorPanel: NSViewRepresentable {
    let title: String
    let content: String
    let saveTitle: String
    let accessibilityPrefix: String
    let isMonospaced: Bool
    let onCancel: () -> Void
    let onSave: (String) -> Bool

    func makeCoordinator() -> Coordinator {
        Coordinator(content: content, onCancel: onCancel, onSave: onSave)
    }

    func makeNSView(context: Context) -> NSStackView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .width
        stack.spacing = 10
        stack.translatesAutoresizingMaskIntoConstraints = false

        let titleLabel = NSTextField(labelWithString: title)
        titleLabel.font = .systemFont(ofSize: NSFont.systemFontSize + 5, weight: .bold)
        titleLabel.setAccessibilityIdentifier("\(accessibilityPrefix)-title")

        let cancelButton = NSButton(
            title: "Cancel",
            target: context.coordinator,
            action: #selector(Coordinator.cancel(_:))
        )
        cancelButton.bezelStyle = .rounded
        cancelButton.controlSize = .small
        cancelButton.setAccessibilityIdentifier("\(accessibilityPrefix)-cancel")

        let saveButton = NSButton(
            title: saveTitle,
            target: context.coordinator,
            action: #selector(Coordinator.save(_:))
        )
        saveButton.bezelStyle = .rounded
        saveButton.controlSize = .small
        saveButton.image = NSImage(systemSymbolName: "checkmark", accessibilityDescription: saveTitle)
        saveButton.imagePosition = .imageLeading
        saveButton.setAccessibilityIdentifier("\(accessibilityPrefix)-save")

        let spacer = NSView()
        spacer.setContentHuggingPriority(.defaultLow, for: .horizontal)
        let header = NSStackView(views: [titleLabel, spacer, cancelButton, saveButton])
        header.orientation = .horizontal
        header.alignment = .centerY
        header.spacing = 8
        header.distribution = .fill

        let textView = NSTextView()
        textView.string = content
        textView.isRichText = false
        textView.allowsUndo = true
        textView.font = isMonospaced ? .monospacedSystemFont(ofSize: NSFont.systemFontSize, weight: .regular) : .systemFont(ofSize: NSFont.systemFontSize)
        textView.drawsBackground = true
        textView.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.45)
        textView.textContainerInset = NSSize(width: 8, height: 8)
        textView.isHorizontallyResizable = false
        textView.isVerticallyResizable = true
        textView.autoresizingMask = [.width]
        textView.textContainer?.widthTracksTextView = true
        textView.delegate = context.coordinator
        textView.setAccessibilityIdentifier("\(accessibilityPrefix)-body")

        let textScroll = NSScrollView()
        textScroll.hasVerticalScroller = true
        textScroll.borderType = .bezelBorder
        textScroll.documentView = textView
        textScroll.translatesAutoresizingMaskIntoConstraints = false
        textScroll.setAccessibilityIdentifier("\(accessibilityPrefix)-body-scroll")
        textScroll.heightAnchor.constraint(equalToConstant: 260).isActive = true

        stack.addArrangedSubview(header)
        stack.addArrangedSubview(textScroll)

        context.coordinator.titleLabel = titleLabel
        context.coordinator.textView = textView
        context.coordinator.loadedContent = content

        return stack
    }

    func updateNSView(_ stack: NSStackView, context: Context) {
        context.coordinator.onCancel = onCancel
        context.coordinator.onSave = onSave
        if context.coordinator.titleLabel?.stringValue != title {
            context.coordinator.titleLabel?.stringValue = title
        }
        if context.coordinator.loadedContent != content, !context.coordinator.hasUserEdited {
            context.coordinator.textView?.string = content
            context.coordinator.loadedContent = content
        }
    }

    final class Coordinator: NSObject, NSTextViewDelegate {
        weak var titleLabel: NSTextField?
        weak var textView: NSTextView?
        var loadedContent: String
        var hasUserEdited = false
        var onCancel: () -> Void
        var onSave: (String) -> Bool

        init(content: String, onCancel: @escaping () -> Void, onSave: @escaping (String) -> Bool) {
            self.loadedContent = content
            self.onCancel = onCancel
            self.onSave = onSave
        }

        func textDidChange(_ notification: Notification) {
            hasUserEdited = true
        }

        @objc func cancel(_ sender: NSButton) {
            onCancel()
        }

        @objc func save(_ sender: NSButton) {
            guard let textView else { return }
            if onSave(textView.string) {
                loadedContent = textView.string
                hasUserEdited = false
            }
        }
    }
}

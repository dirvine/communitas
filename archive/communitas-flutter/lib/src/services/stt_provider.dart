import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:speech_to_text/speech_recognition_error.dart';
import 'package:speech_to_text/speech_recognition_result.dart';
import 'package:speech_to_text/speech_to_text.dart';

import 'canvas_provider.dart';

// ============================================================
// Speech-to-Text State Types
// ============================================================

/// Status of speech recognition
enum SttStatus {
  /// Not initialized
  uninitialized,

  /// Initialized and ready
  ready,

  /// Currently listening
  listening,

  /// Processing final result
  processing,

  /// Speech recognition unavailable
  unavailable,

  /// Error occurred
  error,
}

/// Result from speech recognition
@immutable
class SttResult {
  /// Recognized text
  final String text;

  /// Whether this is a final result (vs interim)
  final bool isFinal;

  /// Confidence score (0.0 to 1.0)
  final double confidence;

  /// Timestamp of recognition
  final DateTime timestamp;

  SttResult({
    required this.text,
    required this.isFinal,
    this.confidence = 0.0,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  SttResult copyWith({
    String? text,
    bool? isFinal,
    double? confidence,
    DateTime? timestamp,
  }) {
    return SttResult(
      text: text ?? this.text,
      isFinal: isFinal ?? this.isFinal,
      confidence: confidence ?? this.confidence,
      timestamp: timestamp ?? this.timestamp,
    );
  }
}

/// State for speech-to-text service
@immutable
class SttState {
  /// Current status
  final SttStatus status;

  /// Current/last recognition result
  final SttResult? currentResult;

  /// Error message if any
  final String? errorMessage;

  /// Available locales for recognition
  final List<LocaleName> availableLocales;

  /// Currently selected locale
  final String selectedLocaleId;

  /// Whether continuous listening is enabled
  final bool continuousListening;

  const SttState({
    this.status = SttStatus.uninitialized,
    this.currentResult,
    this.errorMessage,
    this.availableLocales = const [],
    this.selectedLocaleId = 'en_US',
    this.continuousListening = false,
  });

  SttState copyWith({
    SttStatus? status,
    SttResult? currentResult,
    String? errorMessage,
    List<LocaleName>? availableLocales,
    String? selectedLocaleId,
    bool? continuousListening,
  }) {
    return SttState(
      status: status ?? this.status,
      currentResult: currentResult ?? this.currentResult,
      errorMessage: errorMessage,
      availableLocales: availableLocales ?? this.availableLocales,
      selectedLocaleId: selectedLocaleId ?? this.selectedLocaleId,
      continuousListening: continuousListening ?? this.continuousListening,
    );
  }

  bool get isListening => status == SttStatus.listening;
  bool get isAvailable => status != SttStatus.unavailable && status != SttStatus.uninitialized;
}

// ============================================================
// Speech-to-Text Controller
// ============================================================

/// Controller for speech-to-text functionality
class SttController extends StateNotifier<SttState> {
  final SpeechToText _speech = SpeechToText();

  /// Stream controller for interim results
  final StreamController<SttResult> _resultStreamController =
      StreamController<SttResult>.broadcast();

  /// Stream of recognition results (both interim and final)
  Stream<SttResult> get resultStream => _resultStreamController.stream;

  /// Callback for when a voice command is completed
  void Function(String command)? onVoiceCommand;

  /// Timer for continuous listening restart (cancellable)
  Timer? _continuousRestartTimer;

  SttController() : super(const SttState()) {
    _initialize();
  }

  /// Initialize speech recognition
  Future<void> _initialize() async {
    try {
      final available = await _speech.initialize(
        onStatus: _handleStatus,
        onError: _handleError,
        debugLogging: kDebugMode,
      );

      if (available) {
        final locales = await _speech.locales();
        final systemLocale = await _speech.systemLocale();

        state = state.copyWith(
          status: SttStatus.ready,
          availableLocales: locales,
          selectedLocaleId: systemLocale?.localeId ?? 'en_US',
        );
      } else {
        state = state.copyWith(
          status: SttStatus.unavailable,
          errorMessage: 'Speech recognition not available on this device',
        );
      }
    } catch (e) {
      state = state.copyWith(
        status: SttStatus.error,
        errorMessage: 'Failed to initialize speech recognition: $e',
      );
    }
  }

  /// Start listening for speech
  Future<void> startListening({bool continuous = false}) async {
    if (!state.isAvailable) {
      debugPrint('STT: Cannot start listening - speech recognition unavailable');
      return;
    }

    try {
      state = state.copyWith(
        status: SttStatus.listening,
        continuousListening: continuous,
        currentResult: null,
        errorMessage: null,
      );

      await _speech.listen(
        onResult: _handleResult,
        localeId: state.selectedLocaleId,
        listenOptions: SpeechListenOptions(
          listenMode: continuous ? ListenMode.dictation : ListenMode.confirmation,
          cancelOnError: !continuous,
          partialResults: true,
          autoPunctuation: true,
          enableHapticFeedback: true,
        ),
        pauseFor: continuous ? const Duration(seconds: 3) : const Duration(seconds: 2),
        listenFor: continuous ? const Duration(minutes: 5) : const Duration(seconds: 30),
      );
    } catch (e) {
      state = state.copyWith(
        status: SttStatus.error,
        errorMessage: 'Failed to start listening: $e',
      );
    }
  }

  /// Stop listening
  Future<void> stopListening() async {
    try {
      await _speech.stop();
      state = state.copyWith(
        status: SttStatus.ready,
        continuousListening: false,
      );
    } catch (e) {
      state = state.copyWith(
        status: SttStatus.error,
        errorMessage: 'Failed to stop listening: $e',
      );
    }
  }

  /// Cancel listening (discard results)
  Future<void> cancelListening() async {
    try {
      await _speech.cancel();
      state = state.copyWith(
        status: SttStatus.ready,
        continuousListening: false,
        currentResult: null,
      );
    } catch (e) {
      state = state.copyWith(
        status: SttStatus.error,
        errorMessage: 'Failed to cancel listening: $e',
      );
    }
  }

  /// Change locale
  void setLocale(String localeId) {
    if (state.availableLocales.any((l) => l.localeId == localeId)) {
      state = state.copyWith(selectedLocaleId: localeId);
    }
  }

  /// Handle recognition result
  void _handleResult(SpeechRecognitionResult result) {
    final sttResult = SttResult(
      text: result.recognizedWords,
      isFinal: result.finalResult,
      confidence: result.confidence,
      timestamp: DateTime.now(),
    );

    state = state.copyWith(currentResult: sttResult);
    _resultStreamController.add(sttResult);

    // If final result, trigger voice command callback
    if (result.finalResult && result.recognizedWords.isNotEmpty) {
      onVoiceCommand?.call(result.recognizedWords);

      // If continuous mode, restart listening after a brief pause
      if (state.continuousListening) {
        _cancelContinuousRestartTimer();
        _continuousRestartTimer = Timer(const Duration(milliseconds: 500), () {
          if (state.continuousListening && state.status == SttStatus.ready) {
            startListening(continuous: true);
          }
        });
      }
    }
  }

  /// Cancel the continuous restart timer if active
  void _cancelContinuousRestartTimer() {
    _continuousRestartTimer?.cancel();
    _continuousRestartTimer = null;
  }

  /// Handle status changes
  void _handleStatus(String status) {
    switch (status) {
      case 'listening':
        state = state.copyWith(status: SttStatus.listening);
        break;
      case 'notListening':
        if (state.status == SttStatus.listening) {
          state = state.copyWith(status: SttStatus.processing);
        }
        break;
      case 'done':
        state = state.copyWith(status: SttStatus.ready);
        break;
    }
  }

  /// Handle errors
  void _handleError(SpeechRecognitionError error) {
    if (error.permanent) {
      _cancelContinuousRestartTimer();
      state = state.copyWith(
        status: SttStatus.error,
        errorMessage: 'Speech recognition error: ${error.errorMsg}',
        continuousListening: false,
      );
    } else {
      // Transient error - log and retry if in continuous mode
      debugPrint('STT transient error: ${error.errorMsg}');
      if (state.continuousListening) {
        _cancelContinuousRestartTimer();
        _continuousRestartTimer = Timer(const Duration(seconds: 1), () {
          if (state.continuousListening) {
            startListening(continuous: true);
          }
        });
      }
    }
  }

  @override
  void dispose() {
    _cancelContinuousRestartTimer();
    _resultStreamController.close();
    _speech.cancel();
    super.dispose();
  }
}

// ============================================================
// Providers
// ============================================================

/// Main STT controller provider
final sttControllerProvider =
    StateNotifierProvider<SttController, SttState>((ref) {
  return SttController();
});

/// Provider for STT status
final sttStatusProvider = Provider<SttStatus>((ref) {
  return ref.watch(sttControllerProvider).status;
});

/// Provider for whether STT is currently listening
final isListeningProvider = Provider<bool>((ref) {
  return ref.watch(sttControllerProvider).isListening;
});

/// Provider for current recognition result
final currentSttResultProvider = Provider<SttResult?>((ref) {
  return ref.watch(sttControllerProvider).currentResult;
});

/// Provider for the result stream
final sttResultStreamProvider = StreamProvider<SttResult>((ref) {
  final controller = ref.watch(sttControllerProvider.notifier);
  return controller.resultStream;
});

/// Provider for available locales
final sttLocalesProvider = Provider<List<LocaleName>>((ref) {
  return ref.watch(sttControllerProvider).availableLocales;
});

// ============================================================
// Voice Command Integration
// ============================================================

/// Configuration for voice command handling
@immutable
class VoiceCommandConfig {
  /// Session ID for canvas integration
  final String sessionId;

  /// Prefix words that trigger commands (e.g., "hey canvas")
  final List<String> triggerPhrases;

  /// Whether to show visual feedback
  final bool showVisualFeedback;

  const VoiceCommandConfig({
    required this.sessionId,
    this.triggerPhrases = const ['hey canvas', 'canvas'],
    this.showVisualFeedback = true,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is VoiceCommandConfig &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId &&
          showVisualFeedback == other.showVisualFeedback &&
          _listEquals(triggerPhrases, other.triggerPhrases);

  @override
  int get hashCode => Object.hash(sessionId, showVisualFeedback, Object.hashAll(triggerPhrases));

  static bool _listEquals<T>(List<T> a, List<T> b) {
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}

/// Provider for voice commands with canvas integration
final voiceCommandProvider =
    Provider.family<VoiceCommandHandler, VoiceCommandConfig>((ref, config) {
  return VoiceCommandHandler(ref, config);
});

/// Handles voice commands for canvas interaction
class VoiceCommandHandler {
  final Ref _ref;
  final VoiceCommandConfig _config;

  VoiceCommandHandler(this._ref, this._config) {
    _setupVoiceCommandListener();
  }

  void _setupVoiceCommandListener() {
    final controller = _ref.read(sttControllerProvider.notifier);
    controller.onVoiceCommand = _handleVoiceCommand;
  }

  void _handleVoiceCommand(String command) {
    final lowerCommand = command.toLowerCase().trim();

    // Check for trigger phrases
    String? actualCommand;
    for (final trigger in _config.triggerPhrases) {
      if (lowerCommand.startsWith(trigger)) {
        actualCommand = lowerCommand.substring(trigger.length).trim();
        break;
      }
    }

    // If no trigger phrase, treat entire input as command when in listening mode
    actualCommand ??= lowerCommand;

    if (actualCommand.isEmpty) return;

    // Parse and execute canvas commands
    _executeCanvasCommand(actualCommand);
  }

  void _executeCanvasCommand(String command) {
    // Voice command patterns for canvas interaction
    // These will be processed by the canvas controller

    // Navigation commands
    if (command.contains('zoom in')) {
      _sendCanvasAction('zoom', {'direction': 'in', 'factor': 1.5});
    } else if (command.contains('zoom out')) {
      _sendCanvasAction('zoom', {'direction': 'out', 'factor': 0.67});
    } else if (command.contains('pan left')) {
      _sendCanvasAction('pan', {'dx': -100, 'dy': 0});
    } else if (command.contains('pan right')) {
      _sendCanvasAction('pan', {'dx': 100, 'dy': 0});
    } else if (command.contains('pan up')) {
      _sendCanvasAction('pan', {'dx': 0, 'dy': -100});
    } else if (command.contains('pan down')) {
      _sendCanvasAction('pan', {'dx': 0, 'dy': 100});
    } else if (command.contains('reset view')) {
      _sendCanvasAction('resetView', {});
    }
    // Selection commands
    else if (command.startsWith('select ')) {
      final target = command.substring(7);
      _sendCanvasAction('selectByName', {'name': target});
    } else if (command.contains('deselect') || command.contains('clear selection')) {
      _sendCanvasAction('clearSelection', {});
    }
    // Tool commands
    else if (command.contains('pointer') || command.contains('select tool')) {
      _sendCanvasAction('setTool', {'tool': 'pointer'});
    } else if (command.contains('draw') || command.contains('pen')) {
      _sendCanvasAction('setTool', {'tool': 'draw'});
    } else if (command.contains('text')) {
      _sendCanvasAction('setTool', {'tool': 'text'});
    } else if (command.contains('shape')) {
      _sendCanvasAction('setTool', {'tool': 'shape'});
    }
    // AI/Agent commands (for future integration)
    else if (command.startsWith('ask ') || command.startsWith('tell ')) {
      final message = command.substring(command.indexOf(' ') + 1);
      _sendCanvasAction('agentMessage', {'message': message});
    }
    // Generic command passthrough
    else {
      _sendCanvasAction('voiceCommand', {'command': command});
    }
  }

  void _sendCanvasAction(String action, Map<String, dynamic> params) {
    debugPrint('Voice command: $action with params: $params');

    // Execute canvas commands via the controller
    final canvasController = _ref.read(canvasControllerProvider(_config.sessionId).notifier);

    switch (action) {
      case 'zoom':
        final factor = (params['factor'] as num?)?.toDouble() ?? 1.0;
        canvasController.handleScale(factor, 0, 0);
        break;
      case 'pan':
        final dx = (params['dx'] as num?)?.toDouble() ?? 0.0;
        final dy = (params['dy'] as num?)?.toDouble() ?? 0.0;
        canvasController.handlePanUpdate(dx, dy);
        break;
      case 'resetView':
        canvasController.resetViewport();
        break;
      case 'clearSelection':
        canvasController.clearSelection();
        break;
      case 'selectByName':
        // Selection by name would require element name lookup
        debugPrint('Voice: selectByName not yet implemented');
        break;
      case 'setTool':
        // Tool switching would require tool state management
        debugPrint('Voice: setTool not yet implemented');
        break;
      case 'agentMessage':
        // Agent message would forward to agent handler
        debugPrint('Voice: agentMessage - ${params['message']}');
        break;
      default:
        debugPrint('Voice: Unknown action $action');
    }
  }
}

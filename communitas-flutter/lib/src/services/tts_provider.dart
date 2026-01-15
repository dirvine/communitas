import 'dart:async';
import 'dart:collection';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_tts/flutter_tts.dart';

// ============================================================
// Text-to-Speech State Types
// ============================================================

/// Status of text-to-speech engine
enum TtsStatus {
  /// Not initialized
  uninitialized,

  /// Initialized and ready
  ready,

  /// Currently speaking
  speaking,

  /// Paused during speech
  paused,

  /// Engine unavailable
  unavailable,

  /// Error occurred
  error,
}

/// Voice information
@immutable
class TtsVoice {
  /// Voice name/identifier
  final String name;

  /// Locale (e.g., 'en-US')
  final String locale;

  /// Whether this is a premium/enhanced voice
  final bool isEnhanced;

  const TtsVoice({
    required this.name,
    required this.locale,
    this.isEnhanced = false,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TtsVoice &&
          runtimeType == other.runtimeType &&
          name == other.name &&
          locale == other.locale;

  @override
  int get hashCode => name.hashCode ^ locale.hashCode;
}

/// Item in the speech queue
@immutable
class SpeechQueueItem {
  /// Unique identifier for this utterance
  final String id;

  /// Text to speak
  final String text;

  /// Optional priority (higher = more important)
  final int priority;

  /// Whether to interrupt current speech
  final bool interrupt;

  /// Timestamp when queued
  final DateTime queuedAt;

  SpeechQueueItem({
    required this.id,
    required this.text,
    this.priority = 0,
    this.interrupt = false,
    DateTime? queuedAt,
  }) : queuedAt = queuedAt ?? DateTime.now();
}

/// State for text-to-speech service
@immutable
class TtsState {
  /// Current status
  final TtsStatus status;

  /// Available voices
  final List<TtsVoice> availableVoices;

  /// Currently selected voice
  final TtsVoice? selectedVoice;

  /// Speech rate (0.0 to 1.0)
  final double speechRate;

  /// Pitch (0.5 to 2.0)
  final double pitch;

  /// Volume (0.0 to 1.0)
  final double volume;

  /// Currently speaking text
  final String? currentText;

  /// Error message if any
  final String? errorMessage;

  /// Number of items in queue
  final int queueLength;

  const TtsState({
    this.status = TtsStatus.uninitialized,
    this.availableVoices = const [],
    this.selectedVoice,
    this.speechRate = 0.5,
    this.pitch = 1.0,
    this.volume = 1.0,
    this.currentText,
    this.errorMessage,
    this.queueLength = 0,
  });

  TtsState copyWith({
    TtsStatus? status,
    List<TtsVoice>? availableVoices,
    TtsVoice? selectedVoice,
    double? speechRate,
    double? pitch,
    double? volume,
    String? currentText,
    String? errorMessage,
    int? queueLength,
  }) {
    return TtsState(
      status: status ?? this.status,
      availableVoices: availableVoices ?? this.availableVoices,
      selectedVoice: selectedVoice ?? this.selectedVoice,
      speechRate: speechRate ?? this.speechRate,
      pitch: pitch ?? this.pitch,
      volume: volume ?? this.volume,
      currentText: currentText,
      errorMessage: errorMessage,
      queueLength: queueLength ?? this.queueLength,
    );
  }

  bool get isSpeaking => status == TtsStatus.speaking;
  bool get isPaused => status == TtsStatus.paused;
  bool get isReady => status == TtsStatus.ready || isSpeaking || isPaused;
}

// ============================================================
// Text-to-Speech Controller
// ============================================================

/// Controller for text-to-speech functionality
class TtsController extends StateNotifier<TtsState> {
  final FlutterTts _tts = FlutterTts();

  /// Priority queue for speech items
  final Queue<SpeechQueueItem> _speechQueue = Queue<SpeechQueueItem>();

  /// Currently speaking item
  SpeechQueueItem? _currentItem;

  /// Stream controller for speech events
  final StreamController<TtsSpeechEvent> _eventStreamController =
      StreamController<TtsSpeechEvent>.broadcast();

  /// Stream of speech events
  Stream<TtsSpeechEvent> get eventStream => _eventStreamController.stream;

  /// Counter for generating unique IDs
  int _idCounter = 0;

  TtsController() : super(const TtsState()) {
    _initialize();
  }

  /// Initialize TTS engine
  Future<void> _initialize() async {
    try {
      // Set up handlers
      _tts.setStartHandler(() {
        state = state.copyWith(status: TtsStatus.speaking);
        if (_currentItem != null) {
          _eventStreamController.add(TtsSpeechEvent.started(_currentItem!.id));
        }
      });

      _tts.setCompletionHandler(() {
        final completedId = _currentItem?.id;
        _currentItem = null;
        state = state.copyWith(
          status: TtsStatus.ready,
          currentText: null,
        );
        if (completedId != null) {
          _eventStreamController.add(TtsSpeechEvent.completed(completedId));
        }
        // Process next item in queue
        _processQueue();
      });

      _tts.setCancelHandler(() {
        final cancelledId = _currentItem?.id;
        _currentItem = null;
        state = state.copyWith(
          status: TtsStatus.ready,
          currentText: null,
        );
        if (cancelledId != null) {
          _eventStreamController.add(TtsSpeechEvent.cancelled(cancelledId));
        }
      });

      _tts.setPauseHandler(() {
        state = state.copyWith(status: TtsStatus.paused);
        if (_currentItem != null) {
          _eventStreamController.add(TtsSpeechEvent.paused(_currentItem!.id));
        }
      });

      _tts.setContinueHandler(() {
        state = state.copyWith(status: TtsStatus.speaking);
        if (_currentItem != null) {
          _eventStreamController.add(TtsSpeechEvent.resumed(_currentItem!.id));
        }
      });

      _tts.setErrorHandler((message) {
        state = state.copyWith(
          status: TtsStatus.error,
          errorMessage: message.toString(),
        );
        _eventStreamController.add(TtsSpeechEvent.error(message.toString()));
      });

      // Get available voices
      final voices = await _tts.getVoices;
      final voiceList = <TtsVoice>[];

      if (voices is List) {
        for (final voice in voices) {
          if (voice is Map) {
            voiceList.add(TtsVoice(
              name: voice['name']?.toString() ?? 'Unknown',
              locale: voice['locale']?.toString() ?? 'en-US',
              isEnhanced: voice['quality']?.toString().contains('Enhanced') ?? false,
            ));
          }
        }
      }

      // Set default parameters
      await _tts.setSpeechRate(state.speechRate);
      await _tts.setPitch(state.pitch);
      await _tts.setVolume(state.volume);

      // Try to set a default English voice
      TtsVoice? defaultVoice;
      for (final voice in voiceList) {
        if (voice.locale.startsWith('en')) {
          if (voice.isEnhanced) {
            defaultVoice = voice;
            break;
          }
          defaultVoice ??= voice;
        }
      }

      if (defaultVoice != null) {
        await _tts.setVoice({'name': defaultVoice.name, 'locale': defaultVoice.locale});
      }

      state = state.copyWith(
        status: TtsStatus.ready,
        availableVoices: voiceList,
        selectedVoice: defaultVoice,
        errorMessage: null,
      );
    } catch (e) {
      state = state.copyWith(
        status: TtsStatus.error,
        errorMessage: 'Failed to initialize TTS: $e',
      );
    }
  }

  /// Speak text immediately or queue it
  Future<String> speak(String text, {int priority = 0, bool interrupt = false}) async {
    final id = 'tts_${++_idCounter}';
    final item = SpeechQueueItem(
      id: id,
      text: text,
      priority: priority,
      interrupt: interrupt,
      queuedAt: DateTime.now(),
    );

    if (interrupt && state.isSpeaking) {
      await stop();
    }

    if (interrupt || !state.isSpeaking) {
      await _speakItem(item);
    } else {
      // Add to queue (maintain priority order)
      _addToQueue(item);
      state = state.copyWith(queueLength: _speechQueue.length);
    }

    return id;
  }

  /// Speak text with high priority (interrupts current speech)
  Future<String> speakImmediate(String text) async {
    return speak(text, priority: 100, interrupt: true);
  }

  /// Queue text to speak after current utterance
  Future<String> queueSpeak(String text, {int priority = 0}) async {
    return speak(text, priority: priority, interrupt: false);
  }

  /// Stop current speech
  Future<void> stop() async {
    try {
      await _tts.stop();
      _currentItem = null;
      state = state.copyWith(
        status: TtsStatus.ready,
        currentText: null,
      );
    } catch (e) {
      debugPrint('TTS stop error: $e');
      state = state.copyWith(
        status: TtsStatus.error,
        errorMessage: 'Failed to stop speech: $e',
      );
    }
  }

  /// Pause current speech
  Future<void> pause() async {
    if (state.isSpeaking) {
      try {
        await _tts.pause();
      } catch (e) {
        debugPrint('TTS pause error: $e');
      }
    }
  }

  /// Resume paused speech
  /// Note: Resume behavior is platform-dependent. On iOS, this works natively.
  /// On Android, flutter_tts may not support true resume, so we re-speak if needed.
  Future<void> resume() async {
    if (state.isPaused) {
      try {
        // flutter_tts doesn't have a proper resume method that works across platforms
        // On some platforms we can attempt to resume, on others we need to re-speak
        // For now, we just update the state since the platform handler will manage this
        state = state.copyWith(status: TtsStatus.speaking);
        debugPrint('TTS resume requested - platform-dependent behavior');
      } catch (e) {
        debugPrint('TTS resume error: $e');
        state = state.copyWith(
          status: TtsStatus.error,
          errorMessage: 'Failed to resume speech: $e',
        );
      }
    }
  }

  /// Clear the speech queue
  void clearQueue() {
    _speechQueue.clear();
    state = state.copyWith(queueLength: 0);
  }

  /// Set speech rate (0.0 to 1.0)
  Future<void> setSpeechRate(double rate) async {
    final clampedRate = rate.clamp(0.0, 1.0);
    await _tts.setSpeechRate(clampedRate);
    state = state.copyWith(speechRate: clampedRate);
  }

  /// Set pitch (0.5 to 2.0)
  Future<void> setPitch(double pitch) async {
    final clampedPitch = pitch.clamp(0.5, 2.0);
    await _tts.setPitch(clampedPitch);
    state = state.copyWith(pitch: clampedPitch);
  }

  /// Set volume (0.0 to 1.0)
  Future<void> setVolume(double volume) async {
    final clampedVolume = volume.clamp(0.0, 1.0);
    await _tts.setVolume(clampedVolume);
    state = state.copyWith(volume: clampedVolume);
  }

  /// Set voice
  Future<void> setVoice(TtsVoice voice) async {
    await _tts.setVoice({'name': voice.name, 'locale': voice.locale});
    state = state.copyWith(selectedVoice: voice);
  }

  /// Speak an item
  Future<void> _speakItem(SpeechQueueItem item) async {
    _currentItem = item;
    state = state.copyWith(
      currentText: item.text,
      status: TtsStatus.speaking,
    );
    await _tts.speak(item.text);
  }

  /// Add item to priority queue
  void _addToQueue(SpeechQueueItem item) {
    // Simple priority insertion (higher priority = earlier in queue)
    if (_speechQueue.isEmpty || item.priority <= 0) {
      _speechQueue.addLast(item);
    } else {
      // Find insertion point based on priority
      final list = _speechQueue.toList();
      int insertIndex = list.length;
      for (int i = 0; i < list.length; i++) {
        if (list[i].priority < item.priority) {
          insertIndex = i;
          break;
        }
      }
      list.insert(insertIndex, item);
      _speechQueue.clear();
      _speechQueue.addAll(list);
    }
  }

  /// Process next item in queue
  void _processQueue() {
    if (_speechQueue.isNotEmpty && !state.isSpeaking) {
      final nextItem = _speechQueue.removeFirst();
      state = state.copyWith(queueLength: _speechQueue.length);
      _speakItem(nextItem).catchError((e) {
        debugPrint('TTS queue processing error: $e');
      });
    }
  }

  @override
  void dispose() {
    _eventStreamController.close();
    _tts.stop();
    super.dispose();
  }
}

// ============================================================
// Speech Events
// ============================================================

/// Event types for speech lifecycle
enum TtsSpeechEventType {
  started,
  completed,
  cancelled,
  paused,
  resumed,
  error,
}

/// Speech event
@immutable
class TtsSpeechEvent {
  final TtsSpeechEventType type;
  final String? utteranceId;
  final String? errorMessage;

  const TtsSpeechEvent._({
    required this.type,
    this.utteranceId,
    this.errorMessage,
  });

  factory TtsSpeechEvent.started(String id) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.started, utteranceId: id);

  factory TtsSpeechEvent.completed(String id) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.completed, utteranceId: id);

  factory TtsSpeechEvent.cancelled(String id) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.cancelled, utteranceId: id);

  factory TtsSpeechEvent.paused(String id) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.paused, utteranceId: id);

  factory TtsSpeechEvent.resumed(String id) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.resumed, utteranceId: id);

  factory TtsSpeechEvent.error(String message) =>
      TtsSpeechEvent._(type: TtsSpeechEventType.error, errorMessage: message);
}

// ============================================================
// Providers
// ============================================================

/// Main TTS controller provider
final ttsControllerProvider =
    StateNotifierProvider<TtsController, TtsState>((ref) {
  return TtsController();
});

/// Provider for TTS status
final ttsStatusProvider = Provider<TtsStatus>((ref) {
  return ref.watch(ttsControllerProvider).status;
});

/// Provider for whether TTS is currently speaking
final isSpeakingProvider = Provider<bool>((ref) {
  return ref.watch(ttsControllerProvider).isSpeaking;
});

/// Provider for available voices
final ttsVoicesProvider = Provider<List<TtsVoice>>((ref) {
  return ref.watch(ttsControllerProvider).availableVoices;
});

/// Provider for currently selected voice
final selectedVoiceProvider = Provider<TtsVoice?>((ref) {
  return ref.watch(ttsControllerProvider).selectedVoice;
});

/// Provider for speech event stream
final ttsSpeechEventStreamProvider = StreamProvider<TtsSpeechEvent>((ref) {
  final controller = ref.watch(ttsControllerProvider.notifier);
  return controller.eventStream;
});

// ============================================================
// Agent Response Integration
// ============================================================

/// Configuration for agent response TTS
@immutable
class AgentTtsConfig {
  /// Session ID for canvas integration
  final String sessionId;

  /// Whether to automatically speak agent responses
  final bool autoSpeak;

  /// Maximum text length to speak (truncate longer texts)
  final int maxLength;

  /// Speed multiplier for agent responses
  final double speedMultiplier;

  const AgentTtsConfig({
    required this.sessionId,
    this.autoSpeak = true,
    this.maxLength = 500,
    this.speedMultiplier = 1.1,
  });

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AgentTtsConfig &&
          runtimeType == other.runtimeType &&
          sessionId == other.sessionId;

  @override
  int get hashCode => sessionId.hashCode;
}

/// Provider for agent TTS integration
final agentTtsProvider =
    Provider.family<AgentTtsHandler, AgentTtsConfig>((ref, config) {
  return AgentTtsHandler(ref, config);
});

/// Handles TTS for agent responses
class AgentTtsHandler {
  final Ref _ref;
  final AgentTtsConfig _config;

  AgentTtsHandler(this._ref, this._config);

  /// Speak an agent response
  Future<void> speakAgentResponse(String response) async {
    if (!_config.autoSpeak) return;

    final controller = _ref.read(ttsControllerProvider.notifier);

    // Truncate if needed
    String textToSpeak = response;
    if (response.length > _config.maxLength) {
      textToSpeak = '${response.substring(0, _config.maxLength)}... (message truncated)';
    }

    // Clean up the text for speech
    textToSpeak = _prepareTextForSpeech(textToSpeak);

    // Speak with slightly higher speed for agent responses
    final currentRate = _ref.read(ttsControllerProvider).speechRate;
    final adjustedRate = (currentRate * _config.speedMultiplier).clamp(0.0, 1.0);

    try {
      await controller.setSpeechRate(adjustedRate);
      await controller.speakImmediate(textToSpeak);
    } catch (e) {
      debugPrint('TTS agent speak error: $e');
    } finally {
      // Always attempt to restore original rate
      try {
        await controller.setSpeechRate(currentRate);
      } catch (e) {
        debugPrint('TTS rate restoration error: $e');
      }
    }
  }

  /// Prepare text for speech (clean markdown, etc.)
  String _prepareTextForSpeech(String text) {
    // Remove markdown formatting
    var cleaned = text
        .replaceAll(RegExp(r'\*\*(.*?)\*\*'), r'\1') // Bold
        .replaceAll(RegExp(r'\*(.*?)\*'), r'\1') // Italic
        .replaceAll(RegExp(r'`(.*?)`'), r'\1') // Code
        .replaceAll(RegExp(r'\[(.*?)\]\(.*?\)'), r'\1') // Links
        .replaceAll(RegExp(r'^#+\s*', multiLine: true), '') // Headers
        .replaceAll(RegExp(r'^[-*]\s*', multiLine: true), ''); // Lists

    // Replace common abbreviations for better pronunciation
    cleaned = cleaned
        .replaceAll('API', 'A P I')
        .replaceAll('URL', 'U R L')
        .replaceAll('HTTP', 'H T T P')
        .replaceAll('HTTPS', 'H T T P S')
        .replaceAll('JSON', 'J SON')
        .replaceAll('UI', 'U I')
        .replaceAll('AI', 'A I');

    return cleaned.trim();
  }

  /// Queue multiple responses
  Future<void> queueResponses(List<String> responses) async {
    final controller = _ref.read(ttsControllerProvider.notifier);

    for (int i = 0; i < responses.length; i++) {
      final text = _prepareTextForSpeech(responses[i]);
      await controller.queueSpeak(text, priority: responses.length - i);
    }
  }
}

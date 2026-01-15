import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../core/theme/colors.dart';
import '../services/canvas_provider.dart';

// ============================================================
// Canvas Interaction Models
// ============================================================

/// Type of canvas interaction.
enum InteractionType {
  /// Single tap on the canvas.
  tap,
  
  /// Start of a pan gesture.
  panStart,
  
  /// Update during a pan gesture.
  panUpdate,
  
  /// End of a pan gesture.
  panEnd,
  
  /// Pinch/scale gesture.
  pinch,
  
  /// Voice command interaction.
  voice,
}

/// Represents a user interaction with the canvas.
@immutable
class CanvasInteraction {
  /// The type of interaction.
  final InteractionType type;
  
  /// Position of the interaction in screen coordinates.
  final Offset position;
  
  /// ID of the element interacted with, if any.
  final String? elementId;
  
  /// Voice transcript for voice interactions.
  final String? voiceTranscript;
  
  /// Scale factor for pinch gestures.
  final double? scale;
  
  /// Delta offset for pan gestures.
  final Offset? delta;
  
  /// Timestamp of the interaction.
  final DateTime timestamp;

  CanvasInteraction({
    required this.type,
    required this.position,
    this.elementId,
    this.voiceTranscript,
    this.scale,
    this.delta,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  @override
  String toString() {
    return 'CanvasInteraction(type: $type, position: $position, elementId: $elementId)';
  }
}

// Note: CanvasViewport and CanvasState are imported from canvas_provider.dart
// The controller (SessionCanvasController) manages viewport and selection state

// ============================================================
// Canvas Overlay Widget
// ============================================================

/// A widget that displays a canvas overlay with touch handling
/// and optional video layer compositing.
///
/// This widget integrates with the canvas server via providers
/// and forwards all touch events for collaborative editing.
///
/// Example usage:
/// ```dart
/// CanvasOverlayWidget(
///   sessionId: 'my-session-id',
///   videoLayer: MyWebRTCVideoWidget(),
///   onInteraction: (interaction) {
///     print('Interaction: \$interaction');
///   },
/// )
/// ```
class CanvasOverlayWidget extends ConsumerStatefulWidget {
  /// The canvas session ID to connect to.
  final String sessionId;
  
  /// Optional video layer to composite behind the canvas.
  /// Typically a WebRTC video stream widget.
  final Widget? videoLayer;
  
  /// Callback for canvas interactions.
  /// Called whenever the user interacts with the canvas.
  final void Function(CanvasInteraction)? onInteraction;
  
  /// Whether to show canvas elements.
  /// Set to false to use as a pure gesture overlay.
  final bool showElements;
  
  /// Whether to show a connection indicator.
  final bool showConnectionStatus;
  
  /// Background color when no video layer is provided.
  final Color? backgroundColor;

  const CanvasOverlayWidget({
    super.key,
    required this.sessionId,
    this.videoLayer,
    this.onInteraction,
    this.showElements = true,
    this.showConnectionStatus = true,
    this.backgroundColor,
  });

  @override
  ConsumerState<CanvasOverlayWidget> createState() => _CanvasOverlayWidgetState();
}

class _CanvasOverlayWidgetState extends ConsumerState<CanvasOverlayWidget> {
  // Gesture state
  double _lastScale = 1.0;
  Offset? _lastFocalPoint;
  String? _selectedElementId;
  
  @override
  void initState() {
    super.initState();
    _connectToSession();
  }

  Future<void> _connectToSession() async {
    // The SessionCanvasController auto-connects when created via the family provider.
    // Just reading the provider triggers initialization.
    ref.read(canvasControllerProvider(widget.sessionId));
  }

  // ============================================================
  // Gesture Handlers
  // ============================================================

  void _handleTapDown(TapDownDetails details) {
    final controller = ref.read(canvasControllerProvider(widget.sessionId).notifier);

    // Use controller's handleTap which does hit testing and updates selection
    controller.handleTap(details.localPosition.dx, details.localPosition.dy).then((elementId) {
      _selectedElementId = elementId;

      _notifyInteraction(CanvasInteraction(
        type: InteractionType.tap,
        position: details.localPosition,
        elementId: elementId,
      ));
    });
  }

  void _handleScaleStart(ScaleStartDetails details) {
    _lastScale = 1.0;
    _lastFocalPoint = details.localFocalPoint;
    
    _notifyInteraction(CanvasInteraction(
      type: InteractionType.panStart,
      position: details.localFocalPoint,
    ));
  }

  void _handleScaleUpdate(ScaleUpdateDetails details) {
    final controller = ref.read(canvasControllerProvider(widget.sessionId).notifier);

    // Handle scale (pinch zoom)
    if (details.scale != 1.0) {
      final scaleChange = details.scale / _lastScale;
      _lastScale = details.scale;

      controller.handleScale(
        scaleChange,
        details.localFocalPoint.dx,
        details.localFocalPoint.dy,
      );

      _notifyInteraction(CanvasInteraction(
        type: InteractionType.pinch,
        position: details.localFocalPoint,
        scale: details.scale,
      ));
    }

    // Handle pan
    if (_lastFocalPoint != null) {
      final delta = details.localFocalPoint - _lastFocalPoint!;
      _lastFocalPoint = details.localFocalPoint;

      controller.handlePanUpdate(delta.dx, delta.dy);

      _notifyInteraction(CanvasInteraction(
        type: InteractionType.panUpdate,
        position: details.localFocalPoint,
        delta: delta,
      ));
    }
  }

  void _handleScaleEnd(ScaleEndDetails details) {
    _notifyInteraction(CanvasInteraction(
      type: InteractionType.panEnd,
      position: _lastFocalPoint ?? Offset.zero,
    ));
    
    _lastScale = 1.0;
    _lastFocalPoint = null;
  }

  void _notifyInteraction(CanvasInteraction interaction) {
    widget.onInteraction?.call(interaction);
  }

  // ============================================================
  // Build Methods
  // ============================================================

  @override
  Widget build(BuildContext context) {
    final canvasState = ref.watch(canvasControllerProvider(widget.sessionId));
    final sceneAsync = ref.watch(canvasSceneProvider(widget.sessionId));

    return Stack(
      fit: StackFit.expand,
      children: [
        // Background / Video layer
        _buildBackground(),

        // Canvas elements layer
        if (widget.showElements)
          sceneAsync.when(
            data: (scene) => _buildCanvasLayer(scene, canvasState.viewport),
            loading: () => const SizedBox.shrink(),
            error: (_, __) => const SizedBox.shrink(),
          ),

        // Gesture detection layer (transparent overlay)
        _buildGestureLayer(),

        // Connection status indicator
        if (widget.showConnectionStatus) _buildConnectionIndicator(canvasState.isConnected),
      ],
    );
  }

  Widget _buildBackground() {
    if (widget.videoLayer != null) {
      return widget.videoLayer!;
    }
    
    return Container(
      color: widget.backgroundColor ?? CommunitasColors.deepForest,
    );
  }

  Widget _buildCanvasLayer(CanvasSceneUpdate scene, CanvasViewport viewport) {
    return CustomPaint(
      painter: _CanvasScenePainter(
        sceneData: scene.sceneData,
        viewport: viewport,
        selectedElementId: _selectedElementId,
      ),
      size: Size.infinite,
    );
  }

  Widget _buildGestureLayer() {
    return GestureDetector(
      behavior: HitTestBehavior.translucent,
      onTapDown: _handleTapDown,
      onScaleStart: _handleScaleStart,
      onScaleUpdate: _handleScaleUpdate,
      onScaleEnd: _handleScaleEnd,
      child: Container(
        color: Colors.transparent,
      ),
    );
  }

  Widget _buildConnectionIndicator(bool isConnected) {
    return Positioned(
      top: 16,
      right: 16,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 300),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: isConnected
              ? CommunitasColors.success.withValues(alpha: 0.9)
              : CommunitasColors.warning.withValues(alpha: 0.9),
          borderRadius: BorderRadius.circular(16),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              isConnected ? Icons.cloud_done : Icons.cloud_off,
              size: 16,
              color: CommunitasColors.cream,
            ),
            const SizedBox(width: 6),
            Text(
              isConnected ? 'Connected' : 'Connecting...',
              style: const TextStyle(
                color: CommunitasColors.cream,
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ============================================================
// Canvas Scene Painter
// ============================================================

/// Custom painter for rendering canvas scene elements.
class _CanvasScenePainter extends CustomPainter {
  final Map<String, dynamic>? sceneData;
  final CanvasViewport viewport;
  final String? selectedElementId;

  _CanvasScenePainter({
    required this.sceneData,
    required this.viewport,
    this.selectedElementId,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (sceneData == null) return;

    // Apply viewport transform
    canvas.save();
    canvas.translate(viewport.offsetX, viewport.offsetY);
    canvas.scale(viewport.scale);

    // Draw each element from scene data
    final elements = sceneData!['elements'] as List<dynamic>? ?? [];
    for (final element in elements) {
      if (element is Map<String, dynamic>) {
        _drawElement(canvas, element);
      }
    }

    canvas.restore();
  }

  void _drawElement(Canvas canvas, Map<String, dynamic> element) {
    final id = element['id'] as String? ?? '';
    final type = element['type'] as String? ?? 'unknown';
    final x = (element['x'] as num?)?.toDouble() ?? 0.0;
    final y = (element['y'] as num?)?.toDouble() ?? 0.0;
    final width = (element['width'] as num?)?.toDouble() ?? 100.0;
    final height = (element['height'] as num?)?.toDouble() ?? 100.0;
    final rotation = (element['rotation'] as num?)?.toDouble() ?? 0.0;
    final opacity = (element['opacity'] as num?)?.toDouble() ?? 1.0;
    
    final isSelected = id == selectedElementId;
    
    // Save canvas state for element transform
    canvas.save();
    
    // Apply element rotation around center
    if (rotation != 0) {
      final centerX = x + width / 2;
      final centerY = y + height / 2;
      canvas.translate(centerX, centerY);
      canvas.rotate(rotation);
      canvas.translate(-centerX, -centerY);
    }

    // Draw based on element type
    switch (type) {
      case 'rectangle':
        _drawRectangle(canvas, x, y, width, height, opacity, element);
        break;
      case 'ellipse':
        _drawEllipse(canvas, x, y, width, height, opacity, element);
        break;
      case 'text':
        _drawText(canvas, x, y, width, height, opacity, element);
        break;
      case 'image':
        _drawImagePlaceholder(canvas, x, y, width, height, opacity);
        break;
      case 'video':
        _drawVideoPlaceholder(canvas, x, y, width, height, opacity, element);
        break;
      case 'annotation':
        _drawAnnotation(canvas, x, y, width, height, opacity);
        break;
      default:
        _drawGeneric(canvas, x, y, width, height, opacity);
    }

    // Draw selection border if selected
    if (isSelected) {
      _drawSelectionBorder(canvas, x, y, width, height);
    }

    canvas.restore();
  }

  void _drawRectangle(Canvas canvas, double x, double y, double width, 
      double height, double opacity, Map<String, dynamic> element) {
    final rect = Rect.fromLTWH(x, y, width, height);
    final color = _getElementColor(element);
    final paint = Paint()
      ..color = color.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    canvas.drawRect(rect, paint);
  }

  void _drawEllipse(Canvas canvas, double x, double y, double width, 
      double height, double opacity, Map<String, dynamic> element) {
    final rect = Rect.fromLTWH(x, y, width, height);
    final color = _getElementColor(element);
    final paint = Paint()
      ..color = color.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    canvas.drawOval(rect, paint);
  }

  void _drawText(Canvas canvas, double x, double y, double width, 
      double height, double opacity, Map<String, dynamic> element) {
    final text = element['text'] as String? ?? '';
    final fontSize = (element['fontSize'] as num?)?.toDouble() ?? 16.0;
    
    final textPainter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: CommunitasColors.cream.withValues(alpha: opacity),
          fontSize: fontSize,
        ),
      ),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout(maxWidth: width);
    textPainter.paint(canvas, Offset(x, y));
  }

  void _drawImagePlaceholder(Canvas canvas, double x, double y, 
      double width, double height, double opacity) {
    final rect = Rect.fromLTWH(x, y, width, height);
    final paint = Paint()
      ..color = CommunitasColors.moss.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    canvas.drawRect(rect, paint);
    
    // Draw image icon in center
    final iconPaint = Paint()
      ..color = CommunitasColors.fern.withValues(alpha: opacity)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;
    
    const iconSize = 24.0;
    final iconX = x + (width - iconSize) / 2;
    final iconY = y + (height - iconSize) / 2;
    canvas.drawRect(
      Rect.fromLTWH(iconX, iconY, iconSize, iconSize),
      iconPaint,
    );
  }

  void _drawVideoPlaceholder(Canvas canvas, double x, double y, 
      double width, double height, double opacity, Map<String, dynamic> element) {
    final rect = Rect.fromLTWH(x, y, width, height);
    final paint = Paint()
      ..color = CommunitasColors.deepForest.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    canvas.drawRect(rect, paint);
    
    // Draw video play icon in center
    final iconPaint = Paint()
      ..color = CommunitasColors.jade.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    final centerX = x + width / 2;
    final centerY = y + height / 2;
    const iconSize = 32.0;
    
    final path = Path()
      ..moveTo(centerX - iconSize / 3, centerY - iconSize / 2)
      ..lineTo(centerX + iconSize / 2, centerY)
      ..lineTo(centerX - iconSize / 3, centerY + iconSize / 2)
      ..close();
    
    canvas.drawPath(path, iconPaint);
    
    // Draw label if available
    final label = element['label'] as String?;
    if (label != null && label.isNotEmpty) {
      final labelPainter = TextPainter(
        text: TextSpan(
          text: label,
          style: TextStyle(
            color: CommunitasColors.cream.withValues(alpha: opacity * 0.8),
            fontSize: 12.0,
          ),
        ),
        textDirection: TextDirection.ltr,
      );
      labelPainter.layout(maxWidth: width - 16);
      labelPainter.paint(canvas, Offset(x + 8, y + height - 24));
    }
  }

  void _drawAnnotation(Canvas canvas, double x, double y, 
      double width, double height, double opacity) {
    // Draw annotation marker (pin style)
    final centerX = x + width / 2;
    final bottomY = y + height;
    
    final paint = Paint()
      ..color = CommunitasColors.amber.withValues(alpha: opacity)
      ..style = PaintingStyle.fill;
    
    // Draw circle at top
    canvas.drawCircle(
      Offset(centerX, y + width / 4),
      width / 4,
      paint,
    );
    
    // Draw pointer triangle
    final path = Path()
      ..moveTo(centerX - width / 6, y + width / 3)
      ..lineTo(centerX + width / 6, y + width / 3)
      ..lineTo(centerX, bottomY)
      ..close();
    
    canvas.drawPath(path, paint);
  }

  void _drawGeneric(Canvas canvas, double x, double y, 
      double width, double height, double opacity) {
    final rect = Rect.fromLTWH(x, y, width, height);
    final paint = Paint()
      ..color = CommunitasColors.jade.withValues(alpha: opacity * 0.5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;
    
    canvas.drawRect(rect, paint);
  }

  void _drawSelectionBorder(Canvas canvas, double x, double y, 
      double width, double height) {
    final rect = Rect.fromLTWH(x - 2, y - 2, width + 4, height + 4);
    
    final paint = Paint()
      ..color = CommunitasColors.jade
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;
    
    canvas.drawRect(rect, paint);
    
    // Draw corner handles
    const handleSize = 8.0;
    final handlePaint = Paint()
      ..color = CommunitasColors.jade
      ..style = PaintingStyle.fill;
    
    // Top-left
    canvas.drawRect(
      Rect.fromLTWH(rect.left - handleSize / 2, rect.top - handleSize / 2, 
          handleSize, handleSize),
      handlePaint,
    );
    // Top-right
    canvas.drawRect(
      Rect.fromLTWH(rect.right - handleSize / 2, rect.top - handleSize / 2, 
          handleSize, handleSize),
      handlePaint,
    );
    // Bottom-left
    canvas.drawRect(
      Rect.fromLTWH(rect.left - handleSize / 2, rect.bottom - handleSize / 2, 
          handleSize, handleSize),
      handlePaint,
    );
    // Bottom-right
    canvas.drawRect(
      Rect.fromLTWH(rect.right - handleSize / 2, rect.bottom - handleSize / 2, 
          handleSize, handleSize),
      handlePaint,
    );
  }

  Color _getElementColor(Map<String, dynamic> element) {
    final colorStr = element['color'] as String?;
    if (colorStr != null) {
      try {
        return Color(int.parse(colorStr.replaceFirst('#', '0xFF')));
      } catch (_) {
        // Fall through to default
      }
    }
    return CommunitasColors.jade;
  }

  @override
  bool shouldRepaint(covariant _CanvasScenePainter oldDelegate) {
    return sceneData != oldDelegate.sceneData ||
        viewport != oldDelegate.viewport ||
        selectedElementId != oldDelegate.selectedElementId;
  }
}

// ============================================================
// Video Overlay Compositing
// ============================================================

/// A specialized widget for compositing video with canvas annotations.
///
/// This widget manages the layering of video streams with canvas overlays,
/// supporting multiple video sources and annotation layers.
class CanvasVideoComposite extends ConsumerWidget {
  /// The canvas session ID.
  final String sessionId;
  
  /// Map of video stream widgets by their element IDs.
  final Map<String, Widget> videoStreams;
  
  /// Callback for canvas interactions.
  final void Function(CanvasInteraction)? onInteraction;
  
  /// Whether to show the connection indicator.
  final bool showConnectionStatus;

  const CanvasVideoComposite({
    super.key,
    required this.sessionId,
    required this.videoStreams,
    this.onInteraction,
    this.showConnectionStatus = true,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sceneAsync = ref.watch(canvasSceneProvider(sessionId));
    
    return sceneAsync.when(
      data: (scene) => _buildComposite(context, ref, scene),
      loading: () => _buildLoading(),
      error: (error, stack) => _buildError(error),
    );
  }

  Widget _buildComposite(BuildContext context, WidgetRef ref, CanvasSceneUpdate scene) {
    return Stack(
      fit: StackFit.expand,
      children: [
        // Background
        Container(color: CommunitasColors.deepForest),
        
        // Video streams positioned according to scene data
        ..._buildVideoLayers(scene),
        
        // Canvas overlay for annotations
        CanvasOverlayWidget(
          sessionId: sessionId,
          onInteraction: onInteraction,
          showConnectionStatus: showConnectionStatus,
          backgroundColor: Colors.transparent,
        ),
      ],
    );
  }

  List<Widget> _buildVideoLayers(CanvasSceneUpdate scene) {
    final List<Widget> layers = [];
    final elements = scene.sceneData?['elements'] as List<dynamic>? ?? [];
    
    for (final element in elements) {
      if (element is Map<String, dynamic>) {
        final type = element['type'] as String?;
        final elementId = element['id'] as String?;
        
        if (type == 'video' && elementId != null && videoStreams.containsKey(elementId)) {
          final x = (element['x'] as num?)?.toDouble() ?? 0.0;
          final y = (element['y'] as num?)?.toDouble() ?? 0.0;
          final width = (element['width'] as num?)?.toDouble() ?? 100.0;
          final height = (element['height'] as num?)?.toDouble() ?? 100.0;
          
          layers.add(
            Positioned(
              left: x,
              top: y,
              width: width,
              height: height,
              child: ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: videoStreams[elementId]!,
              ),
            ),
          );
        }
      }
    }
    
    return layers;
  }

  Widget _buildLoading() {
    return const Center(
      child: CircularProgressIndicator(
        color: CommunitasColors.jade,
      ),
    );
  }

  Widget _buildError(Object error) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(
            Icons.error_outline,
            size: 48,
            color: CommunitasColors.error,
          ),
          const SizedBox(height: 16),
          Text(
            'Failed to load canvas',
            style: TextStyle(
              color: CommunitasColors.cream.withValues(alpha: 0.8),
            ),
          ),
        ],
      ),
    );
  }
}

// ============================================================
// Helper Widgets
// ============================================================

/// A widget that displays the canvas toolbar for common actions.
class CanvasToolbar extends ConsumerWidget {
  final String sessionId;
  final VoidCallback? onResetView;
  final VoidCallback? onClearSelection;

  const CanvasToolbar({
    super.key,
    required this.sessionId,
    this.onResetView,
    this.onClearSelection,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final canvasState = ref.watch(canvasControllerProvider(sessionId));

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: CommunitasColors.moss.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Connection status
          Icon(
            canvasState.isConnected ? Icons.cloud_done : Icons.cloud_off,
            size: 16,
            color: canvasState.isConnected ? CommunitasColors.success : CommunitasColors.warning,
          ),
          const SizedBox(width: 8),

          // Reset view button
          IconButton(
            icon: const Icon(Icons.fit_screen, size: 20),
            tooltip: 'Reset View',
            onPressed: () {
              ref.read(canvasControllerProvider(sessionId).notifier).resetViewport();
              onResetView?.call();
            },
            color: CommunitasColors.cream,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            padding: EdgeInsets.zero,
          ),

          // Clear selection button
          IconButton(
            icon: const Icon(Icons.deselect, size: 20),
            tooltip: 'Clear Selection',
            onPressed: () {
              ref.read(canvasControllerProvider(sessionId).notifier).clearSelection();
              onClearSelection?.call();
            },
            color: CommunitasColors.cream,
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            padding: EdgeInsets.zero,
          ),
        ],
      ),
    );
  }
}

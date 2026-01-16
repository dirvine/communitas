import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../bindings/api_exports.dart';
import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';
import '../../../services/ffi_provider.dart';

/// Virtual disk browser with Private/Public/Shared views.
class DriveBrowserScreen extends ConsumerStatefulWidget {
  final String entityType;
  final String entityId;

  const DriveBrowserScreen({
    super.key,
    required this.entityType,
    required this.entityId,
  });

  @override
  ConsumerState<DriveBrowserScreen> createState() => _DriveBrowserScreenState();
}

class _DriveBrowserScreenState extends ConsumerState<DriveBrowserScreen> {
  String _selectedDisk = 'private';
  String _currentPath = '/';

  @override
  Widget build(BuildContext context) {
    return AdaptiveLayout(
      sidebar: const Sidebar(),
      body: Scaffold(
        appBar: AppBar(
          title: const Text('Drive'),
          actions: [
            IconButton(
              icon: const Icon(Icons.upload_file),
              onPressed: _promptCreateFile,
              tooltip: 'Create file',
            ),
            IconButton(
              icon: const Icon(Icons.create_new_folder),
              onPressed: _promptCreateFolder,
              tooltip: 'New folder',
            ),
          ],
        ),
        body: Column(
          children: [
            // Disk type selector
            Container(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  _buildDiskTab('Private', 'private', Icons.lock_outline),
                  const SizedBox(width: 8),
                  _buildDiskTab('Public', 'public', Icons.public),
                  const SizedBox(width: 8),
                  _buildDiskTab('Shared', 'shared', Icons.group_outlined),
                ],
              ),
            ),
            const Divider(height: 1),

            // File list
            Expanded(
              child: _buildFileList(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDiskTab(String label, String value, IconData icon) {
    final isSelected = _selectedDisk == value;

    return Expanded(
      child: InkWell(
        onTap: () => setState(() {
          _selectedDisk = value;
          _currentPath = '/';
        }),
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            color: isSelected ? CommunitasColors.jade : CommunitasColors.moss,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                icon,
                size: 18,
                color: isSelected
                    ? CommunitasColors.cream
                    : CommunitasColors.cream.withOpacity(0.7),
              ),
              const SizedBox(width: 8),
              Text(
                label,
                style: TextStyle(
                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                  color: isSelected
                      ? CommunitasColors.cream
                      : CommunitasColors.cream.withOpacity(0.7),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildFileList() {
    final api = ref.watch(communitasApiProvider);
    if (kIsWeb || api == null) {
      return _buildDemoFileList();
    }

    final diskType = _parseDiskType(_selectedDisk);
    final filesAsync = ref.watch(ffiDiskFilesProvider((
      entityId: widget.entityId,
      diskType: diskType,
      path: _currentPath,
    )));

    return filesAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Center(child: Text('Failed to load files: $e')),
      data: (files) {
        final showUp = _currentPath.isNotEmpty && _currentPath != '/';
        if (files.isEmpty && !showUp) {
          return const Center(child: Text('No files yet'));
        }

        final totalItems = files.length + (showUp ? 1 : 0);
        return ListView.builder(
          padding: const EdgeInsets.all(16),
          itemCount: totalItems,
          itemBuilder: (context, index) {
            if (showUp && index == 0) {
              return _buildUpItem();
            }
            final file = files[index - (showUp ? 1 : 0)];
            return _buildFfiFileItem(file);
          },
        );
      },
    );
  }

  Widget _buildUpItem() {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(8),
      ),
      child: ListTile(
        leading: const Icon(Icons.arrow_upward, color: CommunitasColors.cream),
        title: const Text('..'),
        onTap: _goUp,
      ),
    );
  }

  Widget _buildFfiFileItem(FlutterFileInfo file) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            file.isDirectory ? Icons.folder : _getFileIcon(file.name),
            color: file.isDirectory
                ? CommunitasColors.amber
                : CommunitasColors.jade,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  file.name,
                  style: const TextStyle(fontWeight: FontWeight.w500),
                ),
                Text(
                  file.isDirectory ? 'Folder' : _formatBytes(file.sizeBytes),
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
              ],
            ),
          ),
          PopupMenuButton<String>(
            icon: const Icon(Icons.more_vert, size: 20),
            onSelected: (value) => _handleFileAction(value, file),
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'open', child: Text('Open')),
              if (!file.isDirectory)
                const PopupMenuItem(value: 'download', child: Text('Download')),
              const PopupMenuItem(value: 'delete', child: Text('Delete')),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDemoFileList() {
    final files = [
      _FileItem(name: 'Documents', isFolder: true, size: '5 items'),
      _FileItem(name: 'Images', isFolder: true, size: '12 items'),
      _FileItem(name: 'README.md', isFolder: false, size: '2.4 KB'),
      _FileItem(name: 'project-spec.pdf', isFolder: false, size: '1.2 MB'),
      _FileItem(name: 'meeting-notes.txt', isFolder: false, size: '4.1 KB'),
    ];

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: files.length,
      itemBuilder: (context, index) {
        final file = files[index];
        return _buildFileItem(file);
      },
    );
  }

  Widget _buildFileItem(_FileItem file) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: CommunitasColors.moss,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            file.isFolder ? Icons.folder : _getFileIcon(file.name),
            color: file.isFolder ? CommunitasColors.amber : CommunitasColors.jade,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  file.name,
                  style: const TextStyle(fontWeight: FontWeight.w500),
                ),
                Text(
                  file.size,
                  style: TextStyle(
                    fontSize: 12,
                    color: CommunitasColors.cream.withOpacity(0.5),
                  ),
                ),
              ],
            ),
          ),
          PopupMenuButton(
            icon: const Icon(Icons.more_vert, size: 20),
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'open', child: Text('Open')),
              const PopupMenuItem(value: 'download', child: Text('Download')),
              const PopupMenuItem(value: 'share', child: Text('Share')),
              const PopupMenuItem(value: 'delete', child: Text('Delete')),
            ],
          ),
        ],
      ),
    );
  }

  IconData _getFileIcon(String name) {
    final ext = name.split('.').last.toLowerCase();
    switch (ext) {
      case 'pdf':
        return Icons.picture_as_pdf;
      case 'md':
      case 'txt':
        return Icons.description;
      case 'png':
      case 'jpg':
      case 'jpeg':
        return Icons.image;
      default:
        return Icons.insert_drive_file;
    }
  }

  FlutterDiskType _parseDiskType(String value) {
    switch (value) {
      case 'public':
        return FlutterDiskType.public;
      case 'shared':
        return FlutterDiskType.shared;
      case 'private':
      default:
        return FlutterDiskType.private;
    }
  }

  String _joinPath(String base, String name) {
    if (base.isEmpty || base == '/') {
      return '/$name';
    }
    if (base.endsWith('/')) {
      return '$base$name';
    }
    return '$base/$name';
  }

  String _formatBytes(BigInt bytes) {
    final value = bytes.toDouble();
    const suffixes = ['B', 'KB', 'MB', 'GB', 'TB'];
    var size = value;
    var index = 0;
    while (size >= 1024 && index < suffixes.length - 1) {
      size /= 1024;
      index++;
    }
    return '${size.toStringAsFixed(size >= 10 ? 0 : 1)} ${suffixes[index]}';
  }

  void _goUp() {
    if (_currentPath.isEmpty || _currentPath == '/') return;
    final trimmed = _currentPath.endsWith('/')
        ? _currentPath.substring(0, _currentPath.length - 1)
        : _currentPath;
    final idx = trimmed.lastIndexOf('/');
    setState(() {
      _currentPath = idx <= 0 ? '/' : trimmed.substring(0, idx);
    });
  }

  Future<void> _openFile(FlutterFileInfo file) async {
    final api = ref.read(communitasApiProvider);
    if (api == null) return;
    try {
      final bytes = await api.diskReadFile(
        entityId: widget.entityId,
        diskType: _parseDiskType(_selectedDisk),
        path: file.path,
      );
      final content = utf8.decode(bytes, allowMalformed: true);
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(file.name),
          content: SingleChildScrollView(child: Text(content)),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Close'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Failed to open file: $e')),
      );
    }
  }

  void _openDirectory(FlutterFileInfo file) {
    setState(() {
      _currentPath = file.path;
    });
  }

  Future<void> _promptCreateFolder() async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create folder'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(hintText: 'Folder name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    controller.dispose();
    if (name == null || name.isEmpty) return;

    final controllerNotifier = ref.read(ffiDiskControllerProvider.notifier);
    final path = _joinPath(_currentPath, name);
    final ok = await controllerNotifier.createDirectory(
      entityId: widget.entityId,
      diskType: _parseDiskType(_selectedDisk),
      path: path,
    );
    if (ok) {
      _refreshFiles();
    }
  }

  Future<void> _promptCreateFile() async {
    final nameController = TextEditingController();
    final contentController = TextEditingController();
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create file'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(hintText: 'Filename'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: contentController,
              decoration: const InputDecoration(hintText: 'File contents'),
              maxLines: 6,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Create'),
          ),
        ],
      ),
    );
    if (result != true) {
      nameController.dispose();
      contentController.dispose();
      return;
    }

    final name = nameController.text.trim();
    final content = contentController.text;
    nameController.dispose();
    contentController.dispose();

    if (name.isEmpty) return;
    final controllerNotifier = ref.read(ffiDiskControllerProvider.notifier);
    final path = _joinPath(_currentPath, name);
    final ok = await controllerNotifier.writeFile(
      entityId: widget.entityId,
      diskType: _parseDiskType(_selectedDisk),
      path: path,
      data: utf8.encode(content),
    );
    if (ok) {
      _refreshFiles();
    }
  }

  Future<void> _handleFileAction(String action, FlutterFileInfo file) async {
    switch (action) {
      case 'open':
        if (file.isDirectory) {
          _openDirectory(file);
        } else {
          await _openFile(file);
        }
        break;
      case 'delete':
        await _confirmDelete(file);
        break;
    }
  }

  Future<void> _confirmDelete(FlutterFileInfo file) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Delete ${file.name}?'),
        content: const Text('This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    final controllerNotifier = ref.read(ffiDiskControllerProvider.notifier);
    final ok = await controllerNotifier.deleteFile(
      entityId: widget.entityId,
      diskType: _parseDiskType(_selectedDisk),
      path: file.path,
    );
    if (ok) {
      _refreshFiles();
    }
  }

  void _refreshFiles() {
    ref.invalidate(ffiDiskFilesProvider((
      entityId: widget.entityId,
      diskType: _parseDiskType(_selectedDisk),
      path: _currentPath,
    )));
  }
}

class _FileItem {
  final String name;
  final bool isFolder;
  final String size;

  _FileItem({
    required this.name,
    required this.isFolder,
    required this.size,
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/theme/colors.dart';
import '../../../shared/widgets/sidebar.dart';
import '../../../shared/widgets/adaptive_layout.dart';

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
              onPressed: () {},
              tooltip: 'Upload',
            ),
            IconButton(
              icon: const Icon(Icons.create_new_folder),
              onPressed: () {},
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
        onTap: () => setState(() => _selectedDisk = value),
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
    // Demo files
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
            color: file.isFolder
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

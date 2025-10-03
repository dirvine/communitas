import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { OrganizationView } from './OrganizationView';
import { Box, Typography, CircularProgress } from '@mui/material';
import { Organization } from '../../types/organization';

/**
 * Wrapper component that provides OrganizationView with required props from router params
 */
export const OrganizationViewWrapper: React.FC = () => {
  const { orgId } = useParams<{ orgId: string }>();
  const navigate = useNavigate();
  const { organizations } = useEntityDirectory();

  // Find the organization by ID
  const organization = organizations.find(org => org.id === orgId);

  if (!organization) {
    return (
      <Box sx={{ p: 4, textAlign: 'center' }}>
        <Typography variant="h5" color="text.secondary">
          Organization not found
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          The organization with ID "{orgId}" does not exist.
        </Typography>
      </Box>
    );
  }

  const handleNavigate = (type: string, entity: any) => {
    const routes: Record<string, string> = {
      group: `/group/${entity.id}`,
      project: `/project/${entity.id}`,
      channel: `/channel/${entity.id}`,
      user: `/user/${entity.id}`,
    };

    const route = routes[type];
    if (route) {
      navigate(route);
    }
  };

  const handleCall = (entityType: string, entityId: string, callType: 'voice' | 'video') => {
    console.log(`Initiating ${callType} call with ${entityType} ${entityId}`);
    // TODO: Implement actual call functionality
  };

  // Build hierarchy from organization data
  console.log('OrganizationWrapper - building hierarchy:', {
    orgId: organization.id,
    groups: organization.groups,
    projects: organization.projects,
    users: organization.users
  });

  // Map EntityDirectory organization to proper Organization type
  const organizationWithMembers: Organization = {
    id: organization.id,
    name: organization.name,
    description: organization.description,
    created_at: organization.createdAt ?? new Date(),
    updated_at: organization.updatedAt ?? new Date(),
    owner_id: (organization as any).ownerId ?? '',
    members: (organization as any).members ?? [],
    has_file_system: true,
    storage_quota: {
      allocated_gb: 10,
      used_gb: 0,
      available_gb: 10,
      last_updated: new Date()
    },
    settings: (organization as any).settings ?? {
      visibility: 'private' as const,
      default_member_role: 'Member' as const,
      allow_member_invitations: true,
      require_approval_for_joins: false
    },
    groups: [] as any, // Groups are managed separately
    projects: [] as any // Projects are managed separately
  };

  const groupsWithMembers = (organization.groups ?? []).map(group => ({
    ...group,
    members: (group as any).users ?? [],
  }));

  const projectsWithMembers = (organization.projects ?? []).map(project => ({
    ...project,
    members: (project as any).users ?? [],
  }));

  const hierarchy: any = {
    organization: organizationWithMembers,
    groups: groupsWithMembers,
    projects: projectsWithMembers,
    total_members: organization.users?.length ?? 0,
    total_storage_used_gb: 0, // TODO: Calculate from actual storage data
  };

  console.log('OrganizationWrapper - hierarchy built:', hierarchy);

  const handleRefresh = async () => {
    console.log('Refresh requested - reloading organization data');
    // TODO: Implement refresh functionality
  };

  return (
    <OrganizationView
      organization={organizationWithMembers}
      hierarchy={hierarchy}
      onNavigate={handleNavigate}
      onCall={handleCall}
      onRefresh={handleRefresh}
    />
  );
};

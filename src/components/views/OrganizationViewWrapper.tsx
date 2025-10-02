import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import { OrganizationView } from './OrganizationView';
import { Box, Typography, CircularProgress } from '@mui/material';

/**
 * Wrapper component that provides OrganizationView with required props from router params
 */
export const OrganizationViewWrapper: React.FC = () => {
  const { orgId } = useParams<{ orgId: string }>();
  const navigate = useNavigate();
  const { organizations, refreshDirectory } = useEntityDirectory();

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

  // Map users to members to match the TypeScript interface
  const organizationWithMembers = {
    ...organization,
    members: organization.users ?? [],
  };

  const groupsWithMembers = (organization.groups ?? []).map(group => ({
    ...group,
    members: (group as any).users ?? [],
  }));

  const projectsWithMembers = (organization.projects ?? []).map(project => ({
    ...project,
    members: (project as any).users ?? [],
  }));

  const hierarchy = {
    organization: organizationWithMembers,
    groups: groupsWithMembers,
    projects: projectsWithMembers,
    total_members: organization.users?.length ?? 0,
    total_storage_used_gb: 0, // TODO: Calculate from actual storage data
  };

  console.log('OrganizationWrapper - hierarchy built:', hierarchy);

  return (
    <OrganizationView
      organization={organizationWithMembers}
      hierarchy={hierarchy}
      onNavigate={handleNavigate}
      onCall={handleCall}
      onRefresh={refreshDirectory}
    />
  );
};

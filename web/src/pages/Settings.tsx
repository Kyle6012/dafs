import React, { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Alert,
  Button,
  TextField,
  Divider,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Chip,
} from '@mui/material';
import {
  Link,
  Info,
  Refresh,
  CheckCircle,
  Warning,
} from '@mui/icons-material';
import { config, getBackendUrl } from '../config';
import apiClient from '../api/client';

export const Settings: React.FC = () => {
  const [backendStatus, setBackendStatus] = useState<'checking' | 'online' | 'offline'>('checking');
  const [lastChecked, setLastChecked] = useState<Date | null>(null);

  const checkBackendStatus = async () => {
    setBackendStatus('checking');
    try {
      await apiClient.getSystemStatus();
      setBackendStatus('online');
    } catch (error) {
      setBackendStatus('offline');
    }
    setLastChecked(new Date());
  };

  React.useEffect(() => {
    checkBackendStatus();
  }, []);

  const getStatusColor = () => {
    switch (backendStatus) {
      case 'online': return 'success';
      case 'offline': return 'error';
      case 'checking': return 'warning';
      default: return 'default';
    }
  };

  const getStatusText = () => {
    switch (backendStatus) {
      case 'online': return 'Connected';
      case 'offline': return 'Disconnected';
      case 'checking': return 'Checking...';
      default: return 'Unknown';
    }
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Settings
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Configure your DAFS web dashboard settings
      </Typography>

      {/* Backend Configuration */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Backend Configuration
        </Typography>
        
        <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
          <Link color="primary" sx={{ mr: 1 }} />
          <Typography variant="subtitle1">Backend URL</Typography>
        </Box>
        
        <TextField
          fullWidth
          value={getBackendUrl()}
          label="Backend URL"
          variant="outlined"
          margin="normal"
          InputProps={{
            readOnly: true,
          }}
          helperText="This URL is configured via the VITE_API_URL environment variable"
        />

        <Box sx={{ display: 'flex', alignItems: 'center', mt: 2, mb: 2 }}>
          <Chip
            icon={backendStatus === 'online' ? <CheckCircle /> : <Warning />}
            label={getStatusText()}
            color={getStatusColor() as any}
            sx={{ mr: 2 }}
          />
          <Button
            variant="outlined"
            size="small"
            startIcon={<Refresh />}
            onClick={checkBackendStatus}
            disabled={backendStatus === 'checking'}
          >
            Check Status
          </Button>
        </Box>

        {lastChecked && (
          <Typography variant="body2" color="text.secondary">
            Last checked: {lastChecked.toLocaleString()}
          </Typography>
        )}

        {backendStatus === 'offline' && (
          <Alert severity="warning" sx={{ mt: 2 }}>
            Cannot connect to the backend. Please check:
            <ul>
              <li>The backend server is running</li>
              <li>The URL is correct in your .env file</li>
              <li>There are no firewall issues</li>
            </ul>
          </Alert>
        )}
      </Paper>

      {/* System Information */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          System Information
        </Typography>
        
        <List>
          <ListItem>
            <ListItemIcon>
              <Info />
            </ListItemIcon>
            <ListItemText
              primary="API Timeout"
              secondary={`${config.apiTimeout / 1000} seconds`}
            />
          </ListItem>
          <Divider />
          <ListItem>
            <ListItemIcon>
              <Info />
            </ListItemIcon>
            <ListItemText
              primary="Default Page Size"
              secondary={`${config.defaultPageSize} items`}
            />
          </ListItem>
          <Divider />
          <ListItem>
            <ListItemIcon>
              <Info />
            </ListItemIcon>
            <ListItemText
              primary="Max File Size"
              secondary={`${config.maxFileSize / (1024 * 1024)} MB`}
            />
          </ListItem>
        </List>
      </Paper>

      {/* Feature Flags */}
      <Paper sx={{ p: 3 }}>
        <Typography variant="h6" gutterBottom>
          Feature Status
        </Typography>
        
        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
          <Chip
            label="Real-time Updates"
            color={config.features.enableRealTimeUpdates ? 'success' : 'default'}
            size="small"
          />
          <Chip
            label="File Sharing"
            color={config.features.enableFileSharing ? 'success' : 'default'}
            size="small"
          />
          <Chip
            label="AI Operations"
            color={config.features.enableAI ? 'success' : 'default'}
            size="small"
          />
          <Chip
            label="Peer Management"
            color={config.features.enablePeerManagement ? 'success' : 'default'}
            size="small"
          />
        </Box>
      </Paper>
    </Box>
  );
}; 
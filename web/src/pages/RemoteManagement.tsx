import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Button,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Chip,
  Alert,
  Grid,
  Card,
  CardContent,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  TextareaAutosize,
} from '@mui/material';
import {
  PlayArrow,
  Stop,
  Refresh,
  Settings,
  Terminal,
  Backup,
  Visibility,
  VisibilityOff,
  ExpandMore,
  Computer,
  Memory,
  Speed,
  NetworkCheck,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type {
  RemoteConnection,
  RemoteCommandResult,
  RemoteServiceStatus,
  RemoteLogEntry,
  RemoteConfig,
  BackupInfo,
} from '../types/api';

export const RemoteManagement: React.FC = () => {
  const [connection, setConnection] = useState<RemoteConnection | null>(null);
  const [serviceStatus, setServiceStatus] = useState<RemoteServiceStatus | null>(null);
  const [logs, setLogs] = useState<RemoteLogEntry[]>([]);
  const [configs, setConfigs] = useState<RemoteConfig[]>([]);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [commandHistory, setCommandHistory] = useState<RemoteCommandResult[]>([]);
  const [error, setError] = useState<string>('');

  // Dialogs
  const [connectDialog, setConnectDialog] = useState(false);
  const [commandDialog, setCommandDialog] = useState(false);
  const [configDialog, setConfigDialog] = useState(false);
  const [backupDialog, setBackupDialog] = useState(false);
  const [restoreDialog, setRestoreDialog] = useState(false);

  // Form states
  const [connectForm, setConnectForm] = useState({
    host: '',
    port: 8080,
    username: '',
    password: '',
  });
  const [commandForm, setCommandForm] = useState({
    command: '',
    parameters: '',
  });
  const [configForm, setConfigForm] = useState({
    key: '',
    value: '',
  });
  const [backupForm, setBackupForm] = useState({
    path: '',
  });
  const [restoreForm, setRestoreForm] = useState({
    path: '',
  });
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    if (connection?.is_connected) {
      loadRemoteData();
    }
  }, [connection]);

  const loadRemoteData = async () => {
    if (!connection?.is_connected) return;

    try {
      const [statusData, configsData] = await Promise.all([
        apiClient.getRemoteStatus(),
        apiClient.getRemoteConfig(),
      ]);
      setServiceStatus(statusData);
      setConfigs(configsData);
    } catch (err: any) {
      setError(err.message || 'Failed to load remote data');
    } finally {
    }
  };

  const handleConnect = async () => {
    try {
      const connectionData = await apiClient.connectRemote(
        connectForm.host,
        connectForm.port,
        connectForm.username,
        connectForm.password
      );
      setConnection(connectionData);
      setConnectDialog(false);
      setConnectForm({ host: '', port: 8080, username: '', password: '' });
    } catch (err: any) {
      setError(err.message || 'Failed to connect to remote service');
    }
  };

  const handleExecuteCommand = async () => {
    try {
      const result = await apiClient.executeRemoteCommand(
        commandForm.command,
        commandForm.parameters ? JSON.parse(commandForm.parameters) : undefined
      );
      setCommandHistory([result, ...commandHistory]);
      setCommandDialog(false);
      setCommandForm({ command: '', parameters: '' });
      loadRemoteData();
    } catch (err: any) {
      setError(err.message || 'Failed to execute command');
    }
  };

  const handleServiceAction = async (action: 'start' | 'stop' | 'restart') => {
    try {
      switch (action) {
        case 'start':
          await apiClient.startRemoteService();
          break;
        case 'stop':
          await apiClient.stopRemoteService();
          break;
        case 'restart':
          await apiClient.restartRemoteService();
          break;
      }
      loadRemoteData();
    } catch (err: any) {
      setError(err.message || `Failed to ${action} service`);
    }
  };

  const handleUpdateConfig = async () => {
    try {
      await apiClient.updateRemoteConfig(configForm.key, configForm.value);
      setConfigDialog(false);
      setConfigForm({ key: '', value: '' });
      loadRemoteData();
    } catch (err: any) {
      setError(err.message || 'Failed to update configuration');
    }
  };

  const handleBackup = async () => {
    try {
      const backupInfo = await apiClient.backupRemoteData(backupForm.path);
      setBackups([backupInfo, ...backups]);
      setBackupDialog(false);
      setBackupForm({ path: '' });
    } catch (err: any) {
      setError(err.message || 'Failed to create backup');
    }
  };

  const handleRestore = async () => {
    try {
      await apiClient.restoreRemoteData(restoreForm.path);
      setRestoreDialog(false);
      setRestoreForm({ path: '' });
      loadRemoteData();
    } catch (err: any) {
      setError(err.message || 'Failed to restore backup');
    }
  };

  const handleGetLogs = async (lines = 100) => {
    try {
      const logsData = await apiClient.getRemoteLogs(lines);
      setLogs(logsData);
    } catch (err: any) {
      setError(err.message || 'Failed to get logs');
    }
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'error': return 'error';
      case 'warn': return 'warning';
      case 'info': return 'info';
      default: return 'default';
    }
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Remote Management</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<Terminal />}
            onClick={() => setCommandDialog(true)}
            disabled={!connection?.is_connected}
            sx={{ mr: 1 }}
          >
            Execute Command
          </Button>
          <Button
            variant="outlined"
            startIcon={<Settings />}
            onClick={() => setConfigDialog(true)}
            disabled={!connection?.is_connected}
            sx={{ mr: 1 }}
          >
            Configuration
          </Button>
          <Button
            variant="outlined"
            startIcon={<Backup />}
            onClick={() => setBackupDialog(true)}
            disabled={!connection?.is_connected}
            sx={{ mr: 1 }}
          >
            Backup
          </Button>
          <Button
            variant="outlined"
            startIcon={<Refresh />}
            onClick={loadRemoteData}
            disabled={!connection?.is_connected}
          >
            Refresh
          </Button>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Connection Status */}
      {!connection?.is_connected ? (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Not Connected
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              Connect to a remote DAFS service to manage it
            </Typography>
            <Button
              variant="contained"
              startIcon={<Computer />}
              onClick={() => setConnectDialog(true)}
            >
              Connect to Remote Service
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          {/* Service Status */}
          {serviceStatus && (
            <Grid container spacing={2} sx={{ mb: 3 }}>
              <Grid item xs={12} sm={6} md={3}>
                <Card>
                  <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                      <PlayArrow color={serviceStatus.is_running ? 'success' : 'error'} sx={{ mr: 1 }} />
                      <Typography variant="h6">
                        {serviceStatus.is_running ? 'Running' : 'Stopped'}
                      </Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      Service Status
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Card>
                  <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                      <Speed sx={{ mr: 1 }} />
                      <Typography variant="h6">
                        {formatUptime(serviceStatus.uptime)}
                      </Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      Uptime
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Card>
                  <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                      <Memory sx={{ mr: 1 }} />
                      <Typography variant="h6">
                        {serviceStatus.memory_usage.toFixed(1)}%
                      </Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      Memory Usage
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Card>
                  <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                      <NetworkCheck sx={{ mr: 1 }} />
                      <Typography variant="h6">
                        {serviceStatus.active_connections}
                      </Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      Active Connections
                    </Typography>
                  </CardContent>
                </Card>
              </Grid>
            </Grid>
          )}

          {/* Service Controls */}
          <Paper sx={{ mb: 3 }}>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="h6">Service Controls</Typography>
            </Box>
            <Box sx={{ p: 2 }}>
              <Button
                variant="contained"
                color="success"
                startIcon={<PlayArrow />}
                onClick={() => handleServiceAction('start')}
                sx={{ mr: 1 }}
              >
                Start Service
              </Button>
              <Button
                variant="contained"
                color="error"
                startIcon={<Stop />}
                onClick={() => handleServiceAction('stop')}
                sx={{ mr: 1 }}
              >
                Stop Service
              </Button>
              <Button
                variant="contained"
                color="warning"
                startIcon={<Refresh />}
                onClick={() => handleServiceAction('restart')}
              >
                Restart Service
              </Button>
            </Box>
          </Paper>

          {/* Configuration */}
          <Paper sx={{ mb: 3 }}>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="h6">Configuration</Typography>
            </Box>
            <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Key</TableCell>
                    <TableCell>Value</TableCell>
                    <TableCell>Description</TableCell>
                    <TableCell>Editable</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {configs.map((config) => (
                    <TableRow key={config.key}>
                      <TableCell>{config.key}</TableCell>
                      <TableCell>{config.value}</TableCell>
                      <TableCell>{config.description || '-'}</TableCell>
                      <TableCell>
                        <Chip
                          label={config.is_editable ? 'Yes' : 'No'}
                          color={config.is_editable ? 'success' : 'default'}
                          size="small"
                        />
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Paper>

          {/* Logs */}
          <Paper sx={{ mb: 3 }}>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="h6">Recent Logs</Typography>
              <Button
                size="small"
                onClick={() => handleGetLogs(100)}
                sx={{ ml: 2 }}
              >
                Refresh Logs
              </Button>
            </Box>
            <List sx={{ maxHeight: 400, overflow: 'auto' }}>
              {logs.map((log, index) => (
                <ListItem key={index}>
                  <ListItemIcon>
                    <Chip
                      label={log.level}
                      color={getLogLevelColor(log.level) as any}
                      size="small"
                    />
                  </ListItemIcon>
                  <ListItemText
                    primary={log.message}
                    secondary={`${log.timestamp} - ${log.source}`}
                  />
                </ListItem>
              ))}
            </List>
          </Paper>

          {/* Command History */}
          <Paper>
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Typography variant="h6">Command History</Typography>
            </Box>
            <List>
              {commandHistory.map((result, index) => (
                <Accordion key={index}>
                  <AccordionSummary expandIcon={<ExpandMore />}>
                    <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                      <Chip
                        label={result.success ? 'Success' : 'Failed'}
                        color={result.success ? 'success' : 'error'}
                        size="small"
                        sx={{ mr: 2 }}
                      />
                      <Typography variant="body2" sx={{ flexGrow: 1 }}>
                        {result.output.substring(0, 100)}...
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {result.execution_time}ms
                      </Typography>
                    </Box>
                  </AccordionSummary>
                  <AccordionDetails>
                    <Box>
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        <strong>Output:</strong>
                      </Typography>
                      <TextareaAutosize
                        value={result.output}
                        readOnly
                        style={{
                          width: '100%',
                          minHeight: 100,
                          fontFamily: 'monospace',
                          fontSize: 12,
                          padding: 8,
                          border: '1px solid #ccc',
                          borderRadius: 4,
                        }}
                      />
                      {result.error && (
                        <>
                          <Typography variant="body2" sx={{ mb: 1, mt: 2 }}>
                            <strong>Error:</strong>
                          </Typography>
                          <Typography variant="body2" color="error">
                            {result.error}
                          </Typography>
                        </>
                      )}
                    </Box>
                  </AccordionDetails>
                </Accordion>
              ))}
            </List>
          </Paper>
        </>
      )}

      {/* Connect Dialog */}
      <Dialog open={connectDialog} onClose={() => setConnectDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Connect to Remote Service</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Host"
            value={connectForm.host}
            onChange={(e) => setConnectForm({ ...connectForm, host: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Port"
            type="number"
            value={connectForm.port}
            onChange={(e) => setConnectForm({ ...connectForm, port: parseInt(e.target.value) })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Username"
            value={connectForm.username}
            onChange={(e) => setConnectForm({ ...connectForm, username: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Password"
            type={showPassword ? 'text' : 'password'}
            value={connectForm.password}
            onChange={(e) => setConnectForm({ ...connectForm, password: e.target.value })}
            InputProps={{
              endAdornment: (
                <IconButton onClick={() => setShowPassword(!showPassword)}>
                  {showPassword ? <VisibilityOff /> : <Visibility />}
                </IconButton>
              ),
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConnectDialog(false)}>Cancel</Button>
          <Button onClick={handleConnect} variant="contained">
            Connect
          </Button>
        </DialogActions>
      </Dialog>

      {/* Command Dialog */}
      <Dialog open={commandDialog} onClose={() => setCommandDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>Execute Remote Command</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Command"
            value={commandForm.command}
            onChange={(e) => setCommandForm({ ...commandForm, command: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Parameters (JSON)"
            multiline
            rows={4}
            value={commandForm.parameters}
            onChange={(e) => setCommandForm({ ...commandForm, parameters: e.target.value })}
            placeholder='{"key": "value"}'
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCommandDialog(false)}>Cancel</Button>
          <Button onClick={handleExecuteCommand} variant="contained">
            Execute
          </Button>
        </DialogActions>
      </Dialog>

      {/* Config Dialog */}
      <Dialog open={configDialog} onClose={() => setConfigDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Update Configuration</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Configuration Key"
            value={configForm.key}
            onChange={(e) => setConfigForm({ ...configForm, key: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Configuration Value"
            value={configForm.value}
            onChange={(e) => setConfigForm({ ...configForm, value: e.target.value })}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfigDialog(false)}>Cancel</Button>
          <Button onClick={handleUpdateConfig} variant="contained">
            Update
          </Button>
        </DialogActions>
      </Dialog>

      {/* Backup Dialog */}
      <Dialog open={backupDialog} onClose={() => setBackupDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Backup</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Backup Path"
            value={backupForm.path}
            onChange={(e) => setBackupForm({ ...backupForm, path: e.target.value })}
            sx={{ mt: 1 }}
            placeholder="/path/to/backup"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setBackupDialog(false)}>Cancel</Button>
          <Button onClick={handleBackup} variant="contained">
            Create Backup
          </Button>
        </DialogActions>
      </Dialog>

      {/* Restore Dialog */}
      <Dialog open={restoreDialog} onClose={() => setRestoreDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Restore Backup</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Backup Path"
            value={restoreForm.path}
            onChange={(e) => setRestoreForm({ ...restoreForm, path: e.target.value })}
            sx={{ mt: 1 }}
            placeholder="/path/to/backup"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRestoreDialog(false)}>Cancel</Button>
          <Button onClick={handleRestore} variant="contained" color="warning">
            Restore
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
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
  LinearProgress,
  Grid,
  Card,
  CardContent,
  Avatar,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  ListItemSecondaryAction,
  Divider,
  CircularProgress,
  Tooltip,
} from '@mui/material';
import {
  Search,
  Add,
  Delete,
  Refresh,
  Wifi,
  Computer,
  SignalCellular4Bar,
  SignalCellular0Bar,
  Pin,
  ConnectWithoutContact,
  History,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { Peer, PingResult, BootstrapNode } from '../types/api';

export const PeerDiscovery: React.FC = () => {
  const [peers, setPeers] = useState<Peer[]>([]);
  const [bootstrapNodes, setBootstrapNodes] = useState<BootstrapNode[]>([]);
  const [peerHistory, setPeerHistory] = useState<Peer[]>([]);
  const [loading, setLoading] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string>('');

  // Dialogs
  const [connectDialog, setConnectDialog] = useState(false);
  const [addBootstrapDialog, setAddBootstrapDialog] = useState(false);
  const [pingResults, setPingResults] = useState<Map<string, PingResult>>(new Map());

  // Form states
  const [connectForm, setConnectForm] = useState({
    peerId: '',
    address: '',
  });
  const [bootstrapForm, setBootstrapForm] = useState({
    peerId: '',
    address: '',
    port: 8080,
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [peersData, bootstrapData, historyData] = await Promise.all([
        apiClient.getPeers(),
        apiClient.getBootstrapNodes(),
        apiClient.getPeerHistory(),
      ]);
      setPeers(peersData);
      setBootstrapNodes(bootstrapData);
      setPeerHistory(historyData);
    } catch (err: any) {
      setError(err.message || 'Failed to load peer data');
    } finally {
      setLoading(false);
    }
  };

  const handleDiscoverPeers = async () => {
    try {
      setDiscovering(true);
      const result = await apiClient.discoverPeers();
      setPeers(result.peers);
    } catch (err: any) {
      setError(err.message || 'Failed to discover peers');
    } finally {
      setDiscovering(false);
    }
  };

  const handleScanLocalPeers = async () => {
    try {
      setScanning(true);
      const result = await apiClient.scanLocalPeers();
      setPeers(result.peers);
    } catch (err: any) {
      setError(err.message || 'Failed to scan local peers');
    } finally {
      setScanning(false);
    }
  };

  const handleConnectPeer = async () => {
    try {
      await apiClient.connectPeer({
        peer_id: connectForm.peerId,
        address: connectForm.address || undefined,
      });
      setConnectDialog(false);
      setConnectForm({ peerId: '', address: '' });
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to connect to peer');
    }
  };

  const handlePingPeer = async (peerId: string) => {
    try {
      const result = await apiClient.pingPeer(peerId);
      setPingResults(new Map(pingResults.set(peerId, result)));
    } catch (err: any) {
      setError(err.message || 'Failed to ping peer');
    }
  };

  const handleRemovePeer = async (peerId: string) => {
    try {
      await apiClient.removePeer(peerId);
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to remove peer');
    }
  };

  const handleAddBootstrapNode = async () => {
    try {
      await apiClient.addBootstrapNode({
        peer_id: bootstrapForm.peerId,
        address: bootstrapForm.address,
        port: bootstrapForm.port,
      });
      setAddBootstrapDialog(false);
      setBootstrapForm({ peerId: '', address: '', port: 8080 });
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to add bootstrap node');
    }
  };

  const handleRemoveBootstrapNode = async (peerId: string) => {
    try {
      await apiClient.removeBootstrapNode(peerId);
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to remove bootstrap node');
    }
  };

  const getStatusIcon = (isOnline: boolean) => {
    return isOnline ? (
      <SignalCellular4Bar color="success" />
    ) : (
      <SignalCellular0Bar color="disabled" />
    );
  };

  const getPingColor = (latency?: number) => {
    if (!latency) return 'default';
    if (latency < 50) return 'success';
    if (latency < 100) return 'warning';
    return 'error';
  };

  const formatAddress = (address: string, port: number) => {
    return `${address}:${port}`;
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  if (loading) {
    return (
      <Box sx={{ width: '100%' }}>
        <LinearProgress />
      </Box>
    );
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Peer Discovery</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<Add />}
            onClick={() => setConnectDialog(true)}
            sx={{ mr: 1 }}
          >
            Connect Peer
          </Button>
          <Button
            variant="outlined"
            startIcon={<Add />}
            onClick={() => setAddBootstrapDialog(true)}
            sx={{ mr: 1 }}
          >
            Add Bootstrap
          </Button>
          <Button
            variant="outlined"
            startIcon={<Refresh />}
            onClick={loadData}
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

      {/* Discovery Actions */}
      <Grid container spacing={2} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Network Discovery
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                Discover peers on the network using bootstrap nodes
              </Typography>
              <Button
                variant="contained"
                startIcon={discovering ? <CircularProgress size={20} /> : <Search />}
                onClick={handleDiscoverPeers}
                disabled={discovering}
                fullWidth
              >
                {discovering ? 'Discovering...' : 'Discover Peers'}
              </Button>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Local Network Scan
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                Scan for peers on your local network
              </Typography>
              <Button
                variant="contained"
                startIcon={scanning ? <CircularProgress size={20} /> : <Wifi />}
                onClick={handleScanLocalPeers}
                disabled={scanning}
                fullWidth
              >
                {scanning ? 'Scanning...' : 'Scan Local Network'}
              </Button>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Bootstrap Nodes */}
      <Paper sx={{ mb: 3 }}>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Typography variant="h6">Bootstrap Nodes</Typography>
        </Box>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Peer ID</TableCell>
                <TableCell>Address</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {bootstrapNodes.map((node) => (
                <TableRow key={node.peer_id}>
                  <TableCell>{node.peer_id}</TableCell>
                  <TableCell>{formatAddress(node.address, node.port)}</TableCell>
                  <TableCell>
                    <Chip
                      label={node.is_active ? 'Active' : 'Inactive'}
                      color={node.is_active ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    <IconButton
                      size="small"
                      onClick={() => handleRemoveBootstrapNode(node.peer_id)}
                    >
                      <Delete />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Discovered Peers */}
      <Paper sx={{ mb: 3 }}>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Typography variant="h6">Discovered Peers</Typography>
        </Box>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Peer</TableCell>
                <TableCell>Address</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Files</TableCell>
                <TableCell>Last Seen</TableCell>
                <TableCell>Ping</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {peers.map((peer) => (
                <TableRow key={peer.id}>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <Avatar sx={{ mr: 2 }}>
                        <Computer />
                      </Avatar>
                      {peer.id}
                    </Box>
                  </TableCell>
                  <TableCell>{formatAddress(peer.address, peer.port)}</TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      {getStatusIcon(peer.is_online)}
                      <Chip
                        label={peer.is_online ? 'Online' : 'Offline'}
                        color={peer.is_online ? 'success' : 'default'}
                        size="small"
                        sx={{ ml: 1 }}
                      />
                    </Box>
                  </TableCell>
                  <TableCell>{peer.files_count}</TableCell>
                  <TableCell>{formatDate(peer.last_seen)}</TableCell>
                  <TableCell>
                    {pingResults.has(peer.id) ? (
                      <Chip
                        label={`${pingResults.get(peer.id)?.latency || 'N/A'}ms`}
                        color={getPingColor(pingResults.get(peer.id)?.latency) as any}
                        size="small"
                      />
                    ) : (
                      <IconButton
                        size="small"
                        onClick={() => handlePingPeer(peer.id)}
                      >
                        <Pin />
                      </IconButton>
                    )}
                  </TableCell>
                  <TableCell>
                    <Tooltip title="Connect">
                      <IconButton size="small">
                        <ConnectWithoutContact />
                      </IconButton>
                    </Tooltip>
                    <Tooltip title="Remove">
                      <IconButton
                        size="small"
                        onClick={() => handleRemovePeer(peer.id)}
                      >
                        <Delete />
                      </IconButton>
                    </Tooltip>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Peer History */}
      <Paper>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Typography variant="h6">Connection History</Typography>
        </Box>
        <List>
          {peerHistory.map((peer, index) => (
            <React.Fragment key={peer.id}>
              <ListItem>
                <ListItemAvatar>
                  <Avatar>
                    <History />
                  </Avatar>
                </ListItemAvatar>
                <ListItemText
                  primary={peer.id}
                  secondary={`Last connected: ${formatDate(peer.last_seen)}`}
                />
                <ListItemSecondaryAction>
                  <Chip
                    label={peer.is_online ? 'Online' : 'Offline'}
                    color={peer.is_online ? 'success' : 'default'}
                    size="small"
                  />
                </ListItemSecondaryAction>
              </ListItem>
              {index < peerHistory.length - 1 && <Divider />}
            </React.Fragment>
          ))}
        </List>
      </Paper>

      {/* Connect Peer Dialog */}
      <Dialog open={connectDialog} onClose={() => setConnectDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Connect to Peer</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Peer ID"
            value={connectForm.peerId}
            onChange={(e) => setConnectForm({ ...connectForm, peerId: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Address (optional)"
            value={connectForm.address}
            onChange={(e) => setConnectForm({ ...connectForm, address: e.target.value })}
            placeholder="e.g., 192.168.1.100:8080"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConnectDialog(false)}>Cancel</Button>
          <Button onClick={handleConnectPeer} variant="contained">
            Connect
          </Button>
        </DialogActions>
      </Dialog>

      {/* Add Bootstrap Node Dialog */}
      <Dialog open={addBootstrapDialog} onClose={() => setAddBootstrapDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Add Bootstrap Node</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Peer ID"
            value={bootstrapForm.peerId}
            onChange={(e) => setBootstrapForm({ ...bootstrapForm, peerId: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Address"
            value={bootstrapForm.address}
            onChange={(e) => setBootstrapForm({ ...bootstrapForm, address: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Port"
            type="number"
            value={bootstrapForm.port}
            onChange={(e) => setBootstrapForm({ ...bootstrapForm, port: parseInt(e.target.value) })}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAddBootstrapDialog(false)}>Cancel</Button>
          <Button onClick={handleAddBootstrapNode} variant="contained">
            Add Bootstrap Node
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
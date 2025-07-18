import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Button,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  IconButton,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Alert,
  LinearProgress,
  Card,
  CardContent,
} from '@mui/material';
import {
  Add,
  Delete,
  Refresh,
  Wifi,
  WifiOff,
  Storage,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { Peer, BootstrapNode } from '../types/api';

export const Peers: React.FC = () => {
  const [peers, setPeers] = useState<Peer[]>([]);
  const [bootstrapNodes, setBootstrapNodes] = useState<BootstrapNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [addPeerDialogOpen, setAddPeerDialogOpen] = useState(false);
  const [addBootstrapDialogOpen, setAddBootstrapDialogOpen] = useState(false);
  const [newPeer, setNewPeer] = useState({
    address: '',
    port: '',
    public_key: '',
  });
  const [newBootstrap, setNewBootstrap] = useState({
    peerId: '',
    address: '',
    port: '',
  });

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      setLoading(true);
      const [peersData, bootstrapData] = await Promise.all([
        apiClient.getPeers(),
        apiClient.getBootstrapNodes(),
      ]);
      setPeers(peersData);
      setBootstrapNodes(bootstrapData);
    } catch (err: any) {
      setError(err.message || 'Failed to load peer data');
    } finally {
      setLoading(false);
    }
  };

  const handleAddPeer = async () => {
    try {
      await apiClient.addPeer({
        address: newPeer.address,
        port: parseInt(newPeer.port),
        public_key: newPeer.public_key,
      });
      setAddPeerDialogOpen(false);
      setNewPeer({ address: '', port: '', public_key: '' });
      fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to add peer');
    }
  };

  const handleRemovePeer = async (peerId: string) => {
    if (!window.confirm('Are you sure you want to remove this peer?')) return;

    try {
      await apiClient.removePeer(peerId);
      fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to remove peer');
    }
  };

  const handleAddBootstrap = async () => {
    try {
      await apiClient.addBootstrapNode({
        peer_id: newBootstrap.peerId,
        address: newBootstrap.address,
        port: parseInt(newBootstrap.port),
      });
      setAddBootstrapDialogOpen(false);
      setNewBootstrap({ peerId: '', address: '', port: '' });
      fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to add bootstrap node');
    }
  };

  const handleRemoveBootstrap = async (peerId: string) => {
    if (!window.confirm('Are you sure you want to remove this bootstrap node?')) return;

    try {
      await apiClient.removeBootstrapNode(peerId);
      fetchData();
    } catch (err: any) {
      setError(err.message || 'Failed to remove bootstrap node');
    }
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
        <Typography variant="h4">Peers</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<Refresh />}
            onClick={fetchData}
            sx={{ mr: 1 }}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            startIcon={<Add />}
            onClick={() => setAddPeerDialogOpen(true)}
          >
            Add Peer
          </Button>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Peer Statistics */}
      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2, mb: 3 }}>
        <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Wifi color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Online Peers</Typography>
            </Box>
            <Typography variant="h4">{peers.filter(p => p.is_online).length}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <WifiOff color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Total Peers</Typography>
            </Box>
            <Typography variant="h4">{peers.length}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Storage color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Bootstrap Nodes</Typography>
            </Box>
            <Typography variant="h4">{bootstrapNodes.length}</Typography>
          </CardContent>
        </Card>
      </Box>

      {/* Peers Table */}
      <Paper sx={{ mb: 3 }}>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Address</TableCell>
                <TableCell>Port</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Files</TableCell>
                <TableCell>Last Seen</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {peers.map((peer) => (
                <TableRow key={peer.id}>
                  <TableCell>{peer.address}</TableCell>
                  <TableCell>{peer.port}</TableCell>
                  <TableCell>
                    <Chip
                      label={peer.is_online ? 'Online' : 'Offline'}
                      color={peer.is_online ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{peer.files_count}</TableCell>
                  <TableCell>{new Date(peer.last_seen).toLocaleString()}</TableCell>
                  <TableCell>
                    <IconButton onClick={() => handleRemovePeer(peer.id)}>
                      <Delete />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Bootstrap Nodes */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
        <Typography variant="h5">Bootstrap Nodes</Typography>
        <Button
          variant="outlined"
          startIcon={<Add />}
          onClick={() => setAddBootstrapDialogOpen(true)}
        >
          Add Bootstrap Node
        </Button>
      </Box>

      <Paper>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Address</TableCell>
                <TableCell>Port</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {bootstrapNodes.map((node, index) => (
                <TableRow key={index}>
                  <TableCell>{node.address}</TableCell>
                  <TableCell>{node.port}</TableCell>
                  <TableCell>
                    <Chip
                      label={node.is_active ? 'Active' : 'Inactive'}
                      color={node.is_active ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    <IconButton onClick={() => handleRemoveBootstrap(node.peer_id)}>
                      <Delete />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Add Peer Dialog */}
      <Dialog open={addPeerDialogOpen} onClose={() => setAddPeerDialogOpen(false)}>
        <DialogTitle>Add Peer</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Address"
            margin="normal"
            value={newPeer.address}
            onChange={(e) => setNewPeer({ ...newPeer, address: e.target.value })}
          />
          <TextField
            fullWidth
            label="Port"
            margin="normal"
            type="number"
            value={newPeer.port}
            onChange={(e) => setNewPeer({ ...newPeer, port: e.target.value })}
          />
          <TextField
            fullWidth
            label="Public Key"
            margin="normal"
            multiline
            rows={3}
            value={newPeer.public_key}
            onChange={(e) => setNewPeer({ ...newPeer, public_key: e.target.value })}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAddPeerDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleAddPeer} variant="contained">
            Add Peer
          </Button>
        </DialogActions>
      </Dialog>

      {/* Add Bootstrap Node Dialog */}
      <Dialog open={addBootstrapDialogOpen} onClose={() => setAddBootstrapDialogOpen(false)}>
        <DialogTitle>Add Bootstrap Node</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Peer ID"
            margin="normal"
            value={newBootstrap.peerId}
            onChange={(e) => setNewBootstrap({ ...newBootstrap, peerId: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Address"
            margin="normal"
            value={newBootstrap.address}
            onChange={(e) => setNewBootstrap({ ...newBootstrap, address: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Port"
            margin="normal"
            type="number"
            value={newBootstrap.port}
            onChange={(e) => setNewBootstrap({ ...newBootstrap, port: e.target.value })}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAddBootstrapDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleAddBootstrap} variant="contained">
            Add Bootstrap Node
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
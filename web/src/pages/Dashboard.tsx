import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  Chip,
  LinearProgress,
  Alert,
  Paper,
} from '@mui/material';
import {
  Storage,
  People,
  Psychology,
  TrendingUp,
  CloudUpload,
  Security,
} from '@mui/icons-material';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import apiClient from '../api/client';
import type { Peer, FileInfo } from '../types/api';

interface DashboardStats {
  totalFiles: number;
  totalPeers: number;
  onlinePeers: number;
  totalStorage: number;
  aiTrainingJobs: number;
  recentFiles: FileInfo[];
  recentPeers: Peer[];
  systemStatus: any;
}

export const Dashboard: React.FC = () => {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const { user } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    const fetchDashboardData = async () => {
      try {
        setLoading(true);
        const [peers, files, systemStatus] = await Promise.all([
          apiClient.getPeers(),
          apiClient.getFiles(1, 5),
          apiClient.getSystemStatus(),
        ]);

        setStats({
          totalFiles: files.total,
          totalPeers: peers.length,
          onlinePeers: peers.filter(p => p.is_online).length,
          totalStorage: files.items.reduce((sum, file) => sum + file.size, 0),
          aiTrainingJobs: 0, // TODO: Implement AI job tracking
          recentFiles: files.items,
          recentPeers: peers.slice(0, 5),
          systemStatus,
        });
      } catch (err: any) {
        setError(err.message || 'Failed to load dashboard data');
      } finally {
        setLoading(false);
      }
    };

    fetchDashboardData();
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  if (loading) {
    return (
      <Box sx={{ width: '100%' }}>
        <LinearProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ mb: 2 }}>
        {error}
      </Alert>
    );
  }

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Welcome back, {user?.username}!
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Here's an overview of your DAFS system
      </Typography>

      {stats && (
        <>
          {/* Quick Stats */}
          <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2, mb: 4 }}>
            <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <Storage color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6">Files</Typography>
                </Box>
                <Typography variant="h4">{stats.totalFiles}</Typography>
                <Typography variant="body2" color="text.secondary">
                  {formatBytes(stats.totalStorage)} total
                </Typography>
              </CardContent>
            </Card>

            <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <People color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6">Peers</Typography>
                </Box>
                <Typography variant="h4">{stats.onlinePeers}</Typography>
                <Typography variant="body2" color="text.secondary">
                  {stats.totalPeers} total
                </Typography>
              </CardContent>
            </Card>

            <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <Psychology color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6">AI Jobs</Typography>
                </Box>
                <Typography variant="h4">{stats.aiTrainingJobs}</Typography>
                <Typography variant="body2" color="text.secondary">
                  Active training
                </Typography>
              </CardContent>
            </Card>

            <Card sx={{ flex: '1 1 200px', minWidth: '200px' }}>
              <CardContent>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <TrendingUp color="primary" sx={{ mr: 1 }} />
                  <Typography variant="h6">Status</Typography>
                </Box>
                <Chip
                  label={stats.systemStatus?.status || 'Unknown'}
                  color={stats.systemStatus?.status === 'healthy' ? 'success' : 'warning'}
                  size="small"
                />
              </CardContent>
            </Card>
          </Box>

          {/* Quick Actions */}
          <Paper sx={{ p: 3, mb: 4 }}>
            <Typography variant="h6" gutterBottom>
              Quick Actions
            </Typography>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2 }}>
              <Button
                variant="contained"
                startIcon={<CloudUpload />}
                onClick={() => navigate('/upload')}
              >
                Upload File
              </Button>
              <Button
                variant="outlined"
                startIcon={<People />}
                onClick={() => navigate('/peers')}
              >
                Manage Peers
              </Button>
              <Button
                variant="outlined"
                startIcon={<Psychology />}
                onClick={() => navigate('/ai')}
              >
                AI Operations
              </Button>
              <Button
                variant="outlined"
                startIcon={<Security />}
                onClick={() => navigate('/security')}
              >
                Security Settings
              </Button>
            </Box>
          </Paper>

          {/* Recent Activity */}
          <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 3 }}>
            <Card sx={{ flex: '1 1 400px', minWidth: '400px' }}>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Recent Files
                </Typography>
                {stats.recentFiles.length > 0 ? (
                  stats.recentFiles.map((file) => (
                    <Box key={file.id} sx={{ mb: 2, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
                      <Typography variant="subtitle2">{file.name}</Typography>
                      <Typography variant="body2" color="text.secondary">
                        {formatBytes(file.size)} • {new Date(file.created_at).toLocaleDateString()}
                      </Typography>
                    </Box>
                  ))
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    No recent files
                  </Typography>
                )}
              </CardContent>
            </Card>

            <Card sx={{ flex: '1 1 400px', minWidth: '400px' }}>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Recent Peers
                </Typography>
                {stats.recentPeers.length > 0 ? (
                  stats.recentPeers.map((peer) => (
                    <Box key={peer.id} sx={{ mb: 2, p: 2, bgcolor: 'grey.50', borderRadius: 1 }}>
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <Typography variant="subtitle2">{peer.address}:{peer.port}</Typography>
                        <Chip
                          label={peer.is_online ? 'Online' : 'Offline'}
                          color={peer.is_online ? 'success' : 'default'}
                          size="small"
                        />
                      </Box>
                      <Typography variant="body2" color="text.secondary">
                        {peer.files_count} files • Last seen: {new Date(peer.last_seen).toLocaleDateString()}
                      </Typography>
                    </Box>
                  ))
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    No peers connected
                  </Typography>
                )}
              </CardContent>
            </Card>
          </Box>
        </>
      )}
    </Box>
  );
}; 
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
  FormControlLabel,
  Checkbox,
  Alert,
  LinearProgress,
  Pagination,
  type SelectChangeEvent,
  Autocomplete,
  Tooltip,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
} from '@mui/material';
import {
  Download,
  Delete,
  Share,
  CloudUpload,
  CloudDownload,
  Pause,
  PlayArrow,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { FileInfo, Peer, UploadProgress, DownloadProgress } from '../types/api';

export const Files: React.FC = () => {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [shareDialogOpen, setShareDialogOpen] = useState(false);
  const [p2pDownloadDialogOpen, setP2pDownloadDialogOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null);
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [isPublic, setIsPublic] = useState(false);
  const [uploadTags, setUploadTags] = useState<string[]>([]);
  const [allowedPeers, setAllowedPeers] = useState<string[]>([]);
  const [availableTags, setAvailableTags] = useState<string[]>([]);

  // Upload/Download progress tracking
  const [uploadProgress, setUploadProgress] = useState<Map<string, UploadProgress>>(new Map());
  const [downloadProgress, setDownloadProgress] = useState<Map<string, DownloadProgress>>(new Map());

  // Share dialog state
  const [shareRecipients, setShareRecipients] = useState<string[]>([]);
  const [sharePermissions, setSharePermissions] = useState<'read' | 'write' | 'admin'>('read');

  // P2P download state
  const [selectedPeer, setSelectedPeer] = useState<string>('');

  useEffect(() => {
    fetchFiles();
    fetchPeers();
    fetchAvailableTags();
  }, [page]);

  const fetchFiles = async () => {
    try {
      setLoading(true);
      const response = await apiClient.getFiles(page, 20);
      setFiles(response.items);
      setTotalPages(response.total_pages);
    } catch (err: any) {
      setError(err.message || 'Failed to load files');
    } finally {
      setLoading(false);
    }
  };

  const fetchPeers = async () => {
    try {
      const peersData = await apiClient.getPeers();
      setPeers(peersData);
    } catch (err: any) {
      console.error('Failed to load peers:', err);
    }
  };

  const fetchAvailableTags = async () => {
    // Extract unique tags from existing files
    const tags = new Set<string>();
    files.forEach(file => {
      file.tags.forEach(tag => tags.add(tag));
    });
    setAvailableTags(Array.from(tags));
  };

  const handleUpload = async () => {
    if (!uploadFile) return;

    try {
      await apiClient.uploadFile(uploadFile, isPublic, uploadTags, allowedPeers);
      setUploadDialogOpen(false);
      setUploadFile(null);
      setIsPublic(false);
      setUploadTags([]);
      setAllowedPeers([]);
      fetchFiles();
    } catch (err: any) {
      setError(err.message || 'Failed to upload file');
    }
  };

  const handleDownload = async (file: FileInfo) => {
    try {
      const blob = await apiClient.downloadFile(file.id);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = file.name;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (err: any) {
      setError(err.message || 'Failed to download file');
    }
  };

  const handleP2PDownload = async (file: FileInfo, peerId: string) => {
    try {
      const blob = await apiClient.p2pDownloadFile(file.id, peerId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = file.name;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
      setP2pDownloadDialogOpen(false);
      setSelectedPeer('');
    } catch (err: any) {
      setError(err.message || 'Failed to download file from peer');
    }
  };

  const handleDelete = async (fileId: string) => {
    if (!window.confirm('Are you sure you want to delete this file?')) return;

    try {
      await apiClient.deleteFile(fileId);
      fetchFiles();
    } catch (err: any) {
      setError(err.message || 'Failed to delete file');
    }
  };

  const handleShare = async (fileId: string, userIds: string[], permissions: string) => {
    try {
      await apiClient.shareFile({
        file_id: fileId,
        user_ids: userIds,
        permissions: permissions as 'read' | 'write' | 'admin',
      });
      setShareDialogOpen(false);
      setSelectedFile(null);
      setShareRecipients([]);
      setSharePermissions('read');
    } catch (err: any) {
      setError(err.message || 'Failed to share file');
    }
  };

  const handlePauseUpload = async (fileId: string) => {
    try {
      await apiClient.pauseUpload(fileId);
      // Update progress state
      const progress = uploadProgress.get(fileId);
      if (progress) {
        progress.status = 'paused';
        setUploadProgress(new Map(uploadProgress.set(fileId, progress)));
      }
    } catch (err: any) {
      setError(err.message || 'Failed to pause upload');
    }
  };

  const handleResumeUpload = async (fileId: string) => {
    try {
      await apiClient.resumeUpload(fileId);
      // Update progress state
      const progress = uploadProgress.get(fileId);
      if (progress) {
        progress.status = 'uploading';
        setUploadProgress(new Map(uploadProgress.set(fileId, progress)));
      }
    } catch (err: any) {
      setError(err.message || 'Failed to resume upload');
    }
  };

  const handlePauseDownload = async (fileId: string) => {
    try {
      await apiClient.pauseDownload(fileId);
      // Update progress state
      const progress = downloadProgress.get(fileId);
      if (progress) {
        progress.status = 'paused';
        setDownloadProgress(new Map(downloadProgress.set(fileId, progress)));
      }
    } catch (err: any) {
      setError(err.message || 'Failed to pause download');
    }
  };

  const handleResumeDownload = async (fileId: string) => {
    try {
      await apiClient.resumeDownload(fileId);
      // Update progress state
      const progress = downloadProgress.get(fileId);
      if (progress) {
        progress.status = 'downloading';
        setDownloadProgress(new Map(downloadProgress.set(fileId, progress)));
      }
    } catch (err: any) {
      setError(err.message || 'Failed to resume download');
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  function isUploadProgress(progress: UploadProgress | DownloadProgress): progress is UploadProgress {
    return (progress as UploadProgress).bytes_uploaded !== undefined;
  }

  function isDownloadProgress(progress: UploadProgress | DownloadProgress): progress is DownloadProgress {
    return (progress as DownloadProgress).bytes_downloaded !== undefined;
  }

  const getProgressPercentage = (progress: UploadProgress | DownloadProgress) => {
    if (progress.total_bytes === 0) return 0;
    if (isUploadProgress(progress)) {
      return (progress.bytes_uploaded / progress.total_bytes) * 100;
    } else if (isDownloadProgress(progress)) {
      return (progress.bytes_downloaded / progress.total_bytes) * 100;
    }
    return 0;
  };

  const getProgressColor = (status: string) => {
    switch (status) {
      case 'completed': return 'success';
      case 'failed': return 'error';
      case 'paused': return 'warning';
      default: return 'primary';
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
        <Typography variant="h4">Files</Typography>
        <Button
          variant="contained"
          startIcon={<CloudUpload />}
          onClick={() => setUploadDialogOpen(true)}
        >
          Upload File
        </Button>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Progress Tracking */}
      {uploadProgress.size > 0 && (
        <Paper sx={{ mb: 3 }}>
          <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
            <Typography variant="h6">Upload Progress</Typography>
          </Box>
          <List>
            {Array.from(uploadProgress.values()).map((progress) => (
              <ListItem key={progress.file_id}>
                <ListItemText
                  primary={progress.filename}
                  secondary={`${progress.chunks_uploaded}/${progress.total_chunks} chunks`}
                />
                <ListItemSecondaryAction>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <LinearProgress
                      variant="determinate"
                      value={getProgressPercentage(progress)}
                      color={getProgressColor(progress.status) as any}
                      sx={{ width: 100 }}
                    />
                    <Typography variant="body2">
                      {getProgressPercentage(progress).toFixed(1)}%
                    </Typography>
                    {progress.status === 'uploading' && (
                      <IconButton size="small" onClick={() => handlePauseUpload(progress.file_id)}>
                        <Pause />
                      </IconButton>
                    )}
                    {progress.status === 'paused' && (
                      <IconButton size="small" onClick={() => handleResumeUpload(progress.file_id)}>
                        <PlayArrow />
                      </IconButton>
                    )}
                  </Box>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Paper>
      )}

      {downloadProgress.size > 0 && (
        <Paper sx={{ mb: 3 }}>
          <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
            <Typography variant="h6">Download Progress</Typography>
          </Box>
          <List>
            {Array.from(downloadProgress.values()).map((progress) => (
              <ListItem key={progress.file_id}>
                <ListItemText
                  primary={progress.filename}
                  secondary={`${progress.chunks_downloaded}/${progress.total_chunks} chunks`}
                />
                <ListItemSecondaryAction>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <LinearProgress
                      variant="determinate"
                      value={getProgressPercentage(progress)}
                      color={getProgressColor(progress.status) as any}
                      sx={{ width: 100 }}
                    />
                    <Typography variant="body2">
                      {getProgressPercentage(progress).toFixed(1)}%
                    </Typography>
                    {progress.status === 'downloading' && (
                      <IconButton size="small" onClick={() => handlePauseDownload(progress.file_id)}>
                        <Pause />
                      </IconButton>
                    )}
                    {progress.status === 'paused' && (
                      <IconButton size="small" onClick={() => handleResumeDownload(progress.file_id)}>
                        <PlayArrow />
                      </IconButton>
                    )}
                  </Box>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Paper>
      )}

      <Paper>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell>Size</TableCell>
                <TableCell>Owner</TableCell>
                <TableCell>Tags</TableCell>
                <TableCell>Access Control</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {files.map((file) => (
                <TableRow key={file.id}>
                  <TableCell>{file.name}</TableCell>
                  <TableCell>{formatBytes(file.size)}</TableCell>
                  <TableCell>{file.owner}</TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                      {file.tags.map((tag) => (
                        <Chip key={tag} label={tag} size="small" />
                      ))}
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                      {file.allowed_peers.map((peer) => (
                        <Chip key={peer} label={peer} size="small" color="primary" />
                      ))}
                    </Box>
                  </TableCell>
                  <TableCell>{new Date(file.created_at).toLocaleDateString()}</TableCell>
                  <TableCell>
                    <Chip
                      label={file.is_public ? 'Public' : 'Private'}
                      color={file.is_public ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', gap: 0.5 }}>
                      <Tooltip title="Download">
                        <IconButton onClick={() => handleDownload(file)}>
                          <Download />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="P2P Download">
                        <IconButton onClick={() => {
                          setSelectedFile(file);
                          setP2pDownloadDialogOpen(true);
                        }}>
                          <CloudDownload />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Share">
                        <IconButton onClick={() => {
                          setSelectedFile(file);
                          setShareDialogOpen(true);
                        }}>
                          <Share />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Delete">
                        <IconButton onClick={() => handleDelete(file.id)}>
                          <Delete />
                        </IconButton>
                      </Tooltip>
                    </Box>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
        <Box sx={{ p: 2, display: 'flex', justifyContent: 'center' }}>
          <Pagination
            count={totalPages}
            page={page}
            onChange={(_, newPage) => setPage(newPage)}
          />
        </Box>
      </Paper>

      {/* Upload Dialog */}
      <Dialog open={uploadDialogOpen} onClose={() => setUploadDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Upload File</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            type="file"
            onChange={(e) => setUploadFile((e.target as HTMLInputElement).files?.[0] || null)}
            sx={{ mb: 2, mt: 1 }}
          />
          <FormControlLabel
            control={<Checkbox checked={isPublic} onChange={(e) => setIsPublic(e.target.checked)} />}
            label="Make file public"
            sx={{ mb: 2 }}
          />
          <Autocomplete
            multiple
            freeSolo
            options={availableTags}
            value={uploadTags}
            onChange={(_, newValue) => setUploadTags(newValue)}
            renderInput={(params) => (
              <TextField
                {...params}
                label="Tags"
                placeholder="Add tags..."
                sx={{ mb: 2 }}
              />
            )}
            renderTags={(value, getTagProps) =>
              value.map((option, index) => (
                <Chip
                  label={option}
                  {...getTagProps({ index })}
                  key={option}
                />
              ))
            }
          />
          <Autocomplete
            multiple
            options={peers.map(peer => peer.id)}
            value={allowedPeers}
            onChange={(_, newValue) => setAllowedPeers(newValue)}
            renderInput={(params) => (
              <TextField
                {...params}
                label="Allowed Peers"
                placeholder="Select peers..."
                sx={{ mb: 2 }}
              />
            )}
            renderTags={(value, getTagProps) =>
              value.map((option, index) => (
                <Chip
                  label={option}
                  {...getTagProps({ index })}
                  key={option}
                />
              ))
            }
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setUploadDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleUpload} variant="contained" disabled={!uploadFile}>
            Upload
          </Button>
        </DialogActions>
      </Dialog>

      {/* Share Dialog */}
      <Dialog open={shareDialogOpen} onClose={() => setShareDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Share File</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ mb: 2 }}>
            Share "{selectedFile?.name}" with other users
          </Typography>
          <Autocomplete
            multiple
            options={peers.map(peer => peer.id)}
            value={shareRecipients}
            onChange={(_, newValue) => setShareRecipients(newValue)}
            renderInput={(params) => (
              <TextField
                {...params}
                label="Recipients"
                placeholder="Select users..."
                sx={{ mb: 2 }}
              />
            )}
          />
          <FormControl fullWidth>
            <InputLabel>Permissions</InputLabel>
            <Select
              value={sharePermissions}
              onChange={(e: SelectChangeEvent) => setSharePermissions(e.target.value as 'read' | 'write' | 'admin')}
            >
              <MenuItem value="read">Read Only</MenuItem>
              <MenuItem value="write">Read & Write</MenuItem>
              <MenuItem value="admin">Admin</MenuItem>
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShareDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => selectedFile && handleShare(selectedFile.id, shareRecipients, sharePermissions)}
            variant="contained"
          >
            Share
          </Button>
        </DialogActions>
      </Dialog>

      {/* P2P Download Dialog */}
      <Dialog open={p2pDownloadDialogOpen} onClose={() => setP2pDownloadDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>P2P Download</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ mb: 2 }}>
            Download "{selectedFile?.name}" from a peer
          </Typography>
          <FormControl fullWidth>
            <InputLabel>Select Peer</InputLabel>
            <Select
              value={selectedPeer}
              onChange={(e: SelectChangeEvent) => setSelectedPeer(e.target.value)}
            >
              {peers.map((peer) => (
                <MenuItem key={peer.id} value={peer.id}>
                  {peer.id} ({peer.address}:{peer.port})
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setP2pDownloadDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => selectedFile && selectedPeer && handleP2PDownload(selectedFile, selectedPeer)}
            variant="contained"
            disabled={!selectedPeer}
          >
            Download from Peer
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
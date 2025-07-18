import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Button,
  Paper,
  Card,
  CardContent,
  Alert,
  LinearProgress,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  List,
  ListItem,
  ListItemText,
  Divider,
} from '@mui/material';
import {
  Psychology,
  PlayArrow,
  Refresh,
  TrendingUp,
  Assessment,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { AIRecommendation, AITrainingResult, AIAggregationResult, FileInfo } from '../types/api';

export const AI: React.FC = () => {
  const [recommendations, setRecommendations] = useState<AIRecommendation[]>([]);
  const [trainingResults, setTrainingResults] = useState<AITrainingResult[]>([]);
  const [aggregationResults, setAggregationResults] = useState<AIAggregationResult[]>([]);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [trainingDialogOpen, setTrainingDialogOpen] = useState(false);
  const [aggregationDialogOpen, setAggregationDialogOpen] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [trainingParams, setTrainingParams] = useState('');

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      setLoading(true);
      const [recs, filesData] = await Promise.all([
        apiClient.getAIRecommendations(),
        apiClient.getFiles(1, 100),
      ]);
      setRecommendations(recs);
      setFiles(filesData.items);
    } catch (err: any) {
      setError(err.message || 'Failed to load AI data');
    } finally {
      setLoading(false);
    }
  };

  const handleTrainAI = async () => {
    try {
      const params = trainingParams ? JSON.parse(trainingParams) : {};
      const result = await apiClient.trainAI(params);
      setTrainingResults(prev => [result, ...prev]);
      setTrainingDialogOpen(false);
      setTrainingParams('');
    } catch (err: any) {
      setError(err.message || 'Failed to start AI training');
    }
  };

  const handleAggregate = async () => {
    try {
      const result = await apiClient.aggregateAI(selectedFiles);
      setAggregationResults(prev => [result, ...prev]);
      setAggregationDialogOpen(false);
      setSelectedFiles([]);
    } catch (err: any) {
      setError(err.message || 'Failed to aggregate data');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'success';
      case 'in_progress': return 'warning';
      case 'failed': return 'error';
      default: return 'default';
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
        <Typography variant="h4">AI Operations</Typography>
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
            startIcon={<PlayArrow />}
            onClick={() => setTrainingDialogOpen(true)}
          >
            Train AI
          </Button>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* AI Statistics */}
      <Box sx={{ display: 'flex', gap: 2, mb: 3 }}>
        <Card sx={{ flex: 1 }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Psychology color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Recommendations</Typography>
            </Box>
            <Typography variant="h4">{recommendations.length}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <TrendingUp color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Training Jobs</Typography>
            </Box>
            <Typography variant="h4">{trainingResults.length}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ flex: 1 }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Assessment color="primary" sx={{ mr: 1 }} />
              <Typography variant="h6">Aggregations</Typography>
            </Box>
            <Typography variant="h4">{aggregationResults.length}</Typography>
          </CardContent>
        </Card>
      </Box>

      {/* Recent Recommendations */}
      <Paper sx={{ mb: 3 }}>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Typography variant="h6">Recent Recommendations</Typography>
        </Box>
        <List>
          {recommendations.slice(0, 5).map((rec, index) => (
            <React.Fragment key={rec.id}>
              <ListItem>
                <ListItemText
                  primary={rec.recommendation}
                  secondary={`File: ${rec.file_name} • Confidence: ${(rec.confidence * 100).toFixed(1)}%`}
                />
                <Chip
                  label={`${(rec.confidence * 100).toFixed(1)}%`}
                  color="primary"
                  size="small"
                />
              </ListItem>
              {index < recommendations.length - 1 && <Divider />}
            </React.Fragment>
          ))}
        </List>
      </Paper>

      {/* Training Results */}
      <Paper sx={{ mb: 3 }}>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
          <Typography variant="h6">Training Results</Typography>
        </Box>
        <List>
          {trainingResults.slice(0, 5).map((result, index) => (
            <React.Fragment key={result.id}>
              <ListItem>
                <ListItemText
                  primary={`Training Job ${result.id}`}
                  secondary={`Accuracy: ${(result.accuracy * 100).toFixed(1)}% • Time: ${result.training_time}s`}
                />
                <Chip
                  label={result.status}
                  color={getStatusColor(result.status) as any}
                  size="small"
                />
              </ListItem>
              {index < trainingResults.length - 1 && <Divider />}
            </React.Fragment>
          ))}
        </List>
      </Paper>

      {/* Aggregation Results */}
      <Paper sx={{ mb: 3 }}>
        <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography variant="h6">Aggregation Results</Typography>
          <Button
            variant="outlined"
            size="small"
            onClick={() => setAggregationDialogOpen(true)}
          >
            New Aggregation
          </Button>
        </Box>
        <List>
          {aggregationResults.slice(0, 5).map((result, index) => (
            <React.Fragment key={result.id}>
              <ListItem>
                <ListItemText
                  primary={result.result}
                  secondary={`Confidence: ${(result.confidence * 100).toFixed(1)}% • ${new Date(result.created_at).toLocaleString()}`}
                />
                <Chip
                  label={`${(result.confidence * 100).toFixed(1)}%`}
                  color="primary"
                  size="small"
                />
              </ListItem>
              {index < aggregationResults.length - 1 && <Divider />}
            </React.Fragment>
          ))}
        </List>
      </Paper>

      {/* Training Dialog */}
      <Dialog open={trainingDialogOpen} onClose={() => setTrainingDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Train AI Model</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Training Parameters (JSON)"
            margin="normal"
            multiline
            rows={4}
            value={trainingParams}
            onChange={(e) => setTrainingParams(e.target.value)}
            placeholder='{"epochs": 100, "learning_rate": 0.001}'
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setTrainingDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleTrainAI} variant="contained">
            Start Training
          </Button>
        </DialogActions>
      </Dialog>

      {/* Aggregation Dialog */}
      <Dialog open={aggregationDialogOpen} onClose={() => setAggregationDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Aggregate Data</DialogTitle>
        <DialogContent>
          <Typography variant="body2" sx={{ mb: 2 }}>
            Select files to aggregate:
          </Typography>
          {files.map((file) => (
            <Box key={file.id} sx={{ mb: 1 }}>
              <input
                type="checkbox"
                id={file.id}
                checked={selectedFiles.includes(file.id)}
                onChange={(e) => {
                  if (e.target.checked) {
                    setSelectedFiles([...selectedFiles, file.id]);
                  } else {
                    setSelectedFiles(selectedFiles.filter(id => id !== file.id));
                  }
                }}
              />
              <label htmlFor={file.id} style={{ marginLeft: 8 }}>
                {file.name}
              </label>
            </Box>
          ))}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAggregationDialogOpen(false)}>Cancel</Button>
          <Button 
            onClick={handleAggregate} 
            variant="contained"
            disabled={selectedFiles.length === 0}
          >
            Aggregate
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
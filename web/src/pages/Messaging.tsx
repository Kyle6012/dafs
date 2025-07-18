import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  List,
  ListItemText,
  ListItemAvatar,
  Avatar,
  TextField,
  Button,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  Badge,
  Tabs,
  Tab,
  Alert,
  LinearProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  type SelectChangeEvent,
} from '@mui/material';
import Grid from '@mui/material/Grid';
import ListItem from '@mui/material/ListItem';
import {
  Send,
  Group,
  Person,
  Circle,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { Message, ChatRoom, User, SendMessageRequest } from '../types/api';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`messaging-tabpanel-${index}`}
      aria-labelledby={`messaging-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

export const Messaging: React.FC = () => {
  const [tabValue, setTabValue] = useState(0);
  const [messages, setMessages] = useState<Array<Message>>([]);
  const [rooms, setRooms] = useState<ChatRoom[]>([]);
  const [onlineUsers, setOnlineUsers] = useState<User[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<ChatRoom | null>(null);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [newMessage, setNewMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  // Dialogs
  const [createRoomDialog, setCreateRoomDialog] = useState(false);
  const [newRoomName, setNewRoomName] = useState('');
  const [newRoomParticipants, setNewRoomParticipants] = useState<string[]>([]);
  const [statusDialog, setStatusDialog] = useState(false);
  const [status, setStatus] = useState('online');
  const [statusMessage, setStatusMessage] = useState('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [roomsData, usersData] = await Promise.all([
        apiClient.getRooms(),
        apiClient.getOnlineUsers(),
      ]);
      setRooms(roomsData);
      setOnlineUsers(usersData);
    } catch (err: any) {
      setError(err.message || 'Failed to load messaging data');
    } finally {
      setLoading(false);
    }
  };

  const handleSendMessage = async () => {
    if (!newMessage.trim()) return;

    try {
      const messageData: SendMessageRequest = {
        message: newMessage,
      };

      if (selectedUser) {
        messageData.recipient_id = selectedUser.id;
      } else if (selectedRoom) {
        messageData.room_id = selectedRoom.id;
      }

      await apiClient.sendMessage(messageData);
      setNewMessage('');
      loadMessages();
    } catch (err: any) {
      setError(err.message || 'Failed to send message');
    }
  };

  const loadMessages = async () => {
    if (!selectedUser && !selectedRoom) return;

    try {
      const messagesData = await apiClient.getMessages(
        selectedUser?.id,
        selectedRoom?.id,
        50
      );
      setMessages(messagesData);
    } catch (err: any) {
      setError(err.message || 'Failed to load messages');
    }
  };

  const handleUserSelect = (user: User) => {
    setSelectedUser(user);
    setSelectedRoom(null);
    loadMessages();
  };

  const handleRoomSelect = (room: ChatRoom) => {
    setSelectedRoom(room);
    setSelectedUser(null);
    loadMessages();
  };

  const handleCreateRoom = async () => {
    if (!newRoomName.trim()) return;

    try {
      await apiClient.createRoom(newRoomName, newRoomParticipants);
      setCreateRoomDialog(false);
      setNewRoomName('');
      setNewRoomParticipants([]);
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to create room');
    }
  };

  const handleSetStatus = async () => {
    try {
      await apiClient.setStatus(status, statusMessage);
      setStatusDialog(false);
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to update status');
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'success';
      case 'away': return 'warning';
      case 'busy': return 'error';
      default: return 'default';
    }
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
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
        <Typography variant="h4">Messaging</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<Group />}
            onClick={() => setCreateRoomDialog(true)}
            sx={{ mr: 1 }}
          >
            Create Room
          </Button>
          <Button
            variant="outlined"
            startIcon={<Person />}
            onClick={() => setStatusDialog(true)}
          >
            Set Status
          </Button>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      <Grid container spacing={3}>
        {/* Left Panel - Users and Rooms */}
        <Grid item xs={12} md={4}>
          <Paper>
            <Tabs value={tabValue} onChange={(_, newValue) => setTabValue(newValue)}>
              <Tab label="Direct Messages" />
              <Tab label="Chat Rooms" />
              <Tab label="Online Users" />
            </Tabs>

            <TabPanel value={tabValue} index={0}>
              <List>
                {onlineUsers.map((user) => (
                  <ListItem key={user.id} selected={selectedUser?.id === user.id} onClick={() => handleUserSelect(user)}>
                    <ListItemAvatar>
                      <Badge
                        overlap="circular"
                        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                        badgeContent={
                          <Circle
                            sx={{
                              color: getStatusColor(user.status.status),
                              fontSize: 12,
                            }}
                          />
                        }
                      >
                        <Avatar>{user.username[0].toUpperCase()}</Avatar>
                      </Badge>
                    </ListItemAvatar>
                    <ListItemText
                      primary={user.username}
                      secondary={user.status.message || user.status.status}
                    />
                  </ListItem>
                ))}
              </List>
            </TabPanel>

            <TabPanel value={tabValue} index={1}>
              <List>
                {rooms.map((room) => (
                  <ListItem key={room.id} selected={selectedRoom?.id === room.id} onClick={() => handleRoomSelect(room)}>
                    <ListItemAvatar>
                      <Avatar>
                        <Group />
                      </Avatar>
                    </ListItemAvatar>
                    <ListItemText
                      primary={room.name}
                      secondary={`${room.participants.length} participants`}
                    />
                    {room.unread_count > 0 && (
                      <Badge badgeContent={room.unread_count} color="primary" />
                    )}
                  </ListItem>
                ))}
              </List>
            </TabPanel>

            <TabPanel value={tabValue} index={2}>
              <List>
                {onlineUsers.map((user) => (
                  <ListItem key={user.id}>
                    <ListItemAvatar>
                      <Badge
                        overlap="circular"
                        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                        badgeContent={
                          <Circle
                            sx={{
                              color: getStatusColor(user.status.status),
                              fontSize: 12,
                            }}
                          />
                        }
                      >
                        <Avatar>{user.username[0].toUpperCase()}</Avatar>
                      </Badge>
                    </ListItemAvatar>
                    <ListItemText
                      primary={user.username}
                      secondary={user.status.message || user.status.status}
                    />
                  </ListItem>
                ))}
              </List>
            </TabPanel>
          </Paper>
        </Grid>

        {/* Right Panel - Chat Area */}
        <Grid item xs={12} md={8}>
          <Paper sx={{ height: '70vh', display: 'flex', flexDirection: 'column' }}>
            {/* Chat Header */}
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              {selectedUser && (
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <Avatar sx={{ mr: 2 }}>{selectedUser.username[0].toUpperCase()}</Avatar>
                  <Box>
                    <Typography variant="h6">{selectedUser.username}</Typography>
                    <Chip
                      label={selectedUser.status.status}
                      color={getStatusColor(selectedUser.status.status) as any}
                      size="small"
                    />
                  </Box>
                </Box>
              )}
              {selectedRoom && (
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <Avatar sx={{ mr: 2 }}>
                    <Group />
                  </Avatar>
                  <Box>
                    <Typography variant="h6">{selectedRoom.name}</Typography>
                    <Typography variant="body2" color="text.secondary">
                      {selectedRoom.participants.length} participants
                    </Typography>
                  </Box>
                </Box>
              )}
              {!selectedUser && !selectedRoom && (
                <Typography variant="h6" color="text.secondary">
                  Select a user or room to start messaging
                </Typography>
              )}
            </Box>

            {/* Messages */}
            <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
              {messages.map((message) => (
                <Box
                  key={message.id}
                  sx={{
                    display: 'flex',
                    justifyContent: message.sender_id === 'current_user' ? 'flex-end' : 'flex-start',
                    mb: 1,
                  }}
                >
                  <Paper
                    sx={{
                      p: 1,
                      maxWidth: '70%',
                      backgroundColor: message.sender_id === 'current_user' ? 'primary.main' : 'grey.100',
                      color: message.sender_id === 'current_user' ? 'white' : 'text.primary',
                    }}
                  >
                    <Typography variant="body2">{message.content}</Typography>
                    <Typography variant="caption" sx={{ opacity: 0.7 }}>
                      {formatTime(message.timestamp)}
                    </Typography>
                  </Paper>
                </Box>
              ))}
            </Box>

            {/* Message Input */}
            <Box sx={{ p: 2, borderTop: 1, borderColor: 'divider' }}>
              <Box sx={{ display: 'flex', gap: 1 }}>
                <TextField
                  fullWidth
                  variant="outlined"
                  placeholder="Type a message..."
                  value={newMessage}
                  onChange={(e) => setNewMessage(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
                  disabled={!selectedUser && !selectedRoom}
                />
                <IconButton
                  color="primary"
                  onClick={handleSendMessage}
                  disabled={!newMessage.trim() || (!selectedUser && !selectedRoom)}
                >
                  <Send />
                </IconButton>
              </Box>
            </Box>
          </Paper>
        </Grid>
      </Grid>

      {/* Create Room Dialog */}
      <Dialog open={createRoomDialog} onClose={() => setCreateRoomDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Chat Room</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Room Name"
            value={newRoomName}
            onChange={(e) => setNewRoomName(e.target.value)}
            sx={{ mb: 2, mt: 1 }}
          />
          <FormControl fullWidth>
            <InputLabel>Participants</InputLabel>
            <Select<string[]>
              multiple
              value={newRoomParticipants}
              onChange={(e: SelectChangeEvent<string[]>) => {
                const {
                  target: { value },
                } = e;
                setNewRoomParticipants(
                  typeof value === 'string' ? value.split(',') : value,
                );
              }}
              renderValue={(selected) => (
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                  {(selected as string[]).map((value: string) => (
                    <Chip key={value} label={value} />
                  ))}
                </Box>
              )}
            >
              {onlineUsers.map((user) => (
                <MenuItem key={user.id} value={user.username}>
                  {user.username}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateRoomDialog(false)}>Cancel</Button>
          <Button onClick={handleCreateRoom} variant="contained">
            Create Room
          </Button>
        </DialogActions>
      </Dialog>

      {/* Set Status Dialog */}
      <Dialog open={statusDialog} onClose={() => setStatusDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Set Status</DialogTitle>
        <DialogContent>
          <FormControl fullWidth sx={{ mb: 2, mt: 1 }}>
            <InputLabel>Status</InputLabel>
            <Select
              value={status}
              onChange={(e) => setStatus(e.target.value)}
            >
              <MenuItem value="online">Online</MenuItem>
              <MenuItem value="away">Away</MenuItem>
              <MenuItem value="busy">Busy</MenuItem>
              <MenuItem value="offline">Offline</MenuItem>
            </Select>
          </FormControl>
          <TextField
            fullWidth
            label="Status Message (optional)"
            value={statusMessage}
            onChange={(e) => setStatusMessage(e.target.value)}
            placeholder="e.g., Working on a project"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setStatusDialog(false)}>Cancel</Button>
          <Button onClick={handleSetStatus} variant="contained">
            Set Status
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
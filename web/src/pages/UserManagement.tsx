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
  CardActions,
  Avatar,
  List,
  ListItem,
  ListItemText,
  ListItemAvatar,
  ListItemSecondaryAction,
  Divider,
  InputAdornment,
} from '@mui/material';
import {
  Add,
  Edit,
  Delete,
  Search,
  Computer,
  Visibility,
  VisibilityOff,
  Logout,
  Settings,
} from '@mui/icons-material';
import apiClient from '../api/client';
import type { User, DeviceInfo } from '../types/api';

export const UserManagement: React.FC = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState('');

  // Dialogs
  const [registerDialog, setRegisterDialog] = useState(false);
  const [loginDialog, setLoginDialog] = useState(false);
  const [changeUsernameDialog, setChangeUsernameDialog] = useState(false);
  const [devicesDialog, setDevicesDialog] = useState(false);

  // Form states
  const [registerForm, setRegisterForm] = useState({
    username: '',
    displayName: '',
    email: '',
    password: '',
    confirmPassword: '',
  });
  const [loginForm, setLoginForm] = useState({
    username: '',
    password: '',
  });
  const [newUsername, setNewUsername] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [usersData, currentUserData, devicesData] = await Promise.all([
        apiClient.getAllUsers(),
        apiClient.whoAmI(),
        apiClient.getDevices(),
      ]);
      setUsers(usersData);
      setCurrentUser(currentUserData);
      setDevices(devicesData);
    } catch (err: any) {
      setError(err.message || 'Failed to load user data');
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async () => {
    if (registerForm.password !== registerForm.confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    try {
      await apiClient.registerUser(
        registerForm.username,
        registerForm.displayName,
        registerForm.email || undefined
      );
      setRegisterDialog(false);
      setRegisterForm({
        username: '',
        displayName: '',
        email: '',
        password: '',
        confirmPassword: '',
      });
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to register user');
    }
  };

  const handleLogin = async () => {
    try {
      await apiClient.loginUser(loginForm.username);
      setLoginDialog(false);
      setLoginForm({ username: '', password: '' });
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to login');
    }
  };

  const handleLogoutDevice = async () => {
    try {
      await apiClient.logoutDevice();
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to logout device');
    }
  };

  const handleChangeUsername = async () => {
    try {
      await apiClient.changeUsername(newUsername);
      setChangeUsernameDialog(false);
      setNewUsername('');
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to change username');
    }
  };

  const handleRemoveDevice = async (deviceId: string) => {
    try {
      await apiClient.removeDevice(deviceId);
      loadData();
    } catch (err: any) {
      setError(err.message || 'Failed to remove device');
    }
  };

  const handleSearchUsers = async () => {
    if (!searchQuery.trim()) {
      loadData();
      return;
    }

    try {
      setLoading(true);
      const searchResult = await apiClient.searchUsers(searchQuery);
      setUsers(searchResult.users);
    } catch (err: any) {
      setError(err.message || 'Failed to search users');
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getDeviceStatusColor = (isActive: boolean) => {
    return isActive ? 'success' : 'default';
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
        <Typography variant="h4">User Management</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<Add />}
            onClick={() => setRegisterDialog(true)}
            sx={{ mr: 1 }}
          >
            Register User
          </Button>
          <Button
            variant="outlined"
            startIcon={<Computer />}
            onClick={() => setDevicesDialog(true)}
            sx={{ mr: 1 }}
          >
            My Devices
          </Button>
          <Button
            variant="outlined"
            startIcon={<Settings />}
            onClick={() => setChangeUsernameDialog(true)}
          >
            Change Username
          </Button>
        </Box>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Current User Info */}
      {currentUser && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
              <Avatar sx={{ mr: 2 }}>{currentUser.username[0].toUpperCase()}</Avatar>
              <Box>
                <Typography variant="h6">{currentUser.username}</Typography>
                <Typography variant="body2" color="text.secondary">
                  {currentUser.display_name || 'No display name'}
                </Typography>
              </Box>
            </Box>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2">
                  <strong>Email:</strong> {currentUser.email || 'No email'}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2">
                  <strong>Role:</strong> {currentUser.role}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2">
                  <strong>Created:</strong> {formatDate(currentUser.created_at)}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2">
                  <strong>Devices:</strong> {currentUser.devices.length}
                </Typography>
              </Grid>
            </Grid>
          </CardContent>
          <CardActions>
            <Button
              size="small"
              startIcon={<Logout />}
              onClick={handleLogoutDevice}
            >
              Logout Device
            </Button>
          </CardActions>
        </Card>
      )}

      {/* Search */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Box sx={{ display: 'flex', gap: 2 }}>
          <TextField
            fullWidth
            label="Search users"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearchUsers()}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={handleSearchUsers}>
                    <Search />
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
          <Button
            variant="contained"
            onClick={handleSearchUsers}
            startIcon={<Search />}
          >
            Search
          </Button>
        </Box>
      </Paper>

      {/* Users Table */}
      <Paper>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>User</TableCell>
                <TableCell>Display Name</TableCell>
                <TableCell>Email</TableCell>
                <TableCell>Role</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Devices</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.id}>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <Avatar sx={{ mr: 2 }}>{user.username[0].toUpperCase()}</Avatar>
                      {user.username}
                    </Box>
                  </TableCell>
                  <TableCell>{user.display_name || '-'}</TableCell>
                  <TableCell>{user.email || '-'}</TableCell>
                  <TableCell>
                    <Chip
                      label={user.role}
                      color={user.role === 'admin' ? 'error' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={user.status.status}
                      color={user.status.status === 'online' ? 'success' : 'default'}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{formatDate(user.created_at)}</TableCell>
                  <TableCell>{user.devices.length}</TableCell>
                  <TableCell>
                    <IconButton size="small">
                      <Edit />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Register Dialog */}
      <Dialog open={registerDialog} onClose={() => setRegisterDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Register New User</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Username"
            value={registerForm.username}
            onChange={(e) => setRegisterForm({ ...registerForm, username: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Display Name"
            value={registerForm.displayName}
            onChange={(e) => setRegisterForm({ ...registerForm, displayName: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Email (optional)"
            type="email"
            value={registerForm.email}
            onChange={(e) => setRegisterForm({ ...registerForm, email: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Password"
            type={showPassword ? 'text' : 'password'}
            value={registerForm.password}
            onChange={(e) => setRegisterForm({ ...registerForm, password: e.target.value })}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={() => setShowPassword(!showPassword)}>
                    {showPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
            sx={{ mb: 2 }}
          />
          <TextField
            fullWidth
            label="Confirm Password"
            type={showPassword ? 'text' : 'password'}
            value={registerForm.confirmPassword}
            onChange={(e) => setRegisterForm({ ...registerForm, confirmPassword: e.target.value })}
            error={registerForm.password !== registerForm.confirmPassword && registerForm.confirmPassword !== ''}
            helperText={
              registerForm.password !== registerForm.confirmPassword && registerForm.confirmPassword !== ''
                ? 'Passwords do not match'
                : ''
            }
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRegisterDialog(false)}>Cancel</Button>
          <Button onClick={handleRegister} variant="contained">
            Register
          </Button>
        </DialogActions>
      </Dialog>

      {/* Login Dialog */}
      <Dialog open={loginDialog} onClose={() => setLoginDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Login</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="Username"
            value={loginForm.username}
            onChange={(e) => setLoginForm({ ...loginForm, username: e.target.value })}
            sx={{ mb: 2, mt: 1 }}
          />
          <TextField
            fullWidth
            label="Password"
            type={showPassword ? 'text' : 'password'}
            value={loginForm.password}
            onChange={(e) => setLoginForm({ ...loginForm, password: e.target.value })}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={() => setShowPassword(!showPassword)}>
                    {showPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setLoginDialog(false)}>Cancel</Button>
          <Button onClick={handleLogin} variant="contained">
            Login
          </Button>
        </DialogActions>
      </Dialog>

      {/* Change Username Dialog */}
      <Dialog open={changeUsernameDialog} onClose={() => setChangeUsernameDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Change Username</DialogTitle>
        <DialogContent>
          <TextField
            fullWidth
            label="New Username"
            value={newUsername}
            onChange={(e) => setNewUsername(e.target.value)}
            sx={{ mt: 1 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setChangeUsernameDialog(false)}>Cancel</Button>
          <Button onClick={handleChangeUsername} variant="contained">
            Change Username
          </Button>
        </DialogActions>
      </Dialog>

      {/* Devices Dialog */}
      <Dialog open={devicesDialog} onClose={() => setDevicesDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>My Devices</DialogTitle>
        <DialogContent>
          <List>
            {devices.map((device) => (
              <React.Fragment key={device.id}>
                <ListItem>
                  <ListItemAvatar>
                    <Avatar>
                      <Computer />
                    </Avatar>
                  </ListItemAvatar>
                  <ListItemText
                    primary={device.name}
                    secondary={`Last seen: ${formatDate(device.last_seen)}`}
                  />
                  <ListItemSecondaryAction>
                    <Chip
                      label={device.is_active ? 'Active' : 'Inactive'}
                      color={getDeviceStatusColor(device.is_active) as any}
                      size="small"
                      sx={{ mr: 1 }}
                    />
                    <IconButton
                      edge="end"
                      onClick={() => handleRemoveDevice(device.id)}
                      disabled={device.is_active}
                    >
                      <Delete />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
                <Divider />
              </React.Fragment>
            ))}
          </List>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDevicesDialog(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}; 
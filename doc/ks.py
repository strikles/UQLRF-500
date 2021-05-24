import numpy as np

sample_rate = 44100
damping = 0.999

def generate(f, vol, nsamples):
    """Generate a Karplus-Strong pluck.

    Arguments:
    f -- the frequency, as an integer
    vol -- volume, a float in [0.0, 1.0]
    nsamples -- the number of samples to generate. To generate a pluck t
    seconds long, one needs t * sample_rate samples. nsamples is an int.

    Return value:
    A numpy array of floats with length nsamples.
    """

    N = sample_rate // f
    buf = np.random.rand(N) * 2 - 1
    samples = np.empty(nsamples, dtype=float)
    print(len(samples), N)
    for i in range(nsamples):
        samples[i] = buf[i % N]
        avg = damping * 0.5 * (buf[i % N] + buf[(1 + i) % N])
        buf[i % N] = avg

    return samples * vol


def generate_chord(f, nsamples):
    """Generate a chord of several plucks

    Arguments:
    f -- a (usually short) list of integer frequencies
    nsamples -- the length of the chord, a chord of length t seconds needs
    t * sample_rate, an integer.

    Return value:
    A numpy array
    """
    samples = np.zeros(nsamples, dtype=float)
    print(len(samples), samples)
    for note in f:
        samples += generate(note, 0.5, nsamples)
        print(len(samples))
    print(len(samples), samples)
    return samples


if __name__ == "__main__":
    import matplotlib.pyplot as plt
    from scipy.io.wavfile import write

    strum = generate_chord(  # A Major
        [220, 220 * 5 // 4, 220 * 6 // 4, 220 * 2], sample_rate * 5)
    plt.plot(strum)
    plt.show()

    write("pluck.wav", sample_rate, strum)
    
    strum = generate_chord(  # A Major
        [220 * 2], sample_rate * 5)
    plt.plot(strum)
    plt.show()

    write("pluckA4.wav", sample_rate, strum)
